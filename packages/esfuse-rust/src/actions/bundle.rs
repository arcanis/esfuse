extern crate queues;

use parcel_sourcemap::SourceMap;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

use crate::{CompilationError, utils};
use crate::Project;
use crate::transforms::OnTransformSwcOpts;
use crate::types::*;

#[derive(Debug)]
struct BundleMessage {
  locator: ModuleLocator,
  sender: UnboundedSender<BundleMessage>,
}

struct BundleModule {
  locator: ModuleLocator,
  mime_type: String,
  code: String,
  map: Option<SourceMap>,
  newlines: usize,
  resolutions: HashMap<String, Option<String>>,
}

impl BundleModule {
  fn new(locator: ModuleLocator, transform: OnTransformResultData, resolutions: HashMap<String, Option<String>>) -> BundleModule {
    let map = transform.map.map(|str| {
      parcel_sourcemap::SourceMap::from_json("/", &str)
        .expect("Assertion failed: Expected the SWC-generated sourcemap to be readable")
    });

    let newlines = count_newlines(transform.code.as_str());

    Self {
      locator,
      mime_type: transform.mime_type,
      code: transform.code,
      map,
      newlines,
      resolutions,
    }
  }
}

#[derive(Serialize)]
struct BundleMeta {
  error: Option<CompilationError>,
  path: Option<String>,
  resolutions: HashMap<String, Option<String>>,
}

pub async fn bundle(project_base: Arc<Project>, args: OnBundleArgs) -> OnBundleResult {
  let project = project_base.as_ref();

  let mut final_nl_count = 0_usize;
  let mut final_source = String::default();
  let mut final_source_map = parcel_sourcemap::SourceMap::new("");

  if let Some(runtime_locator) = &args.opts.runtime {
    let runtime_res = super::transform::transform(project, OnTransformArgs {
      locator: runtime_locator.clone(),
      opts: Default::default(),
    }).await;

    match runtime_res.result {
      Ok(runtime) => {
        final_nl_count += count_newlines(&runtime.code);
        final_source.push_str(&runtime.code);
        final_source.push(' ');
      },

      Err(err) => {
        return OnBundleResult {
          result: Err(err),
          dependencies: vec![],
        };
      },
    }
  };

  let build_results_container
    = Arc::new(Mutex::new(HashMap::new()));

  let (tx, mut rx)
    = tokio::sync::mpsc::unbounded_channel();

  tx.send(BundleMessage {
    locator: args.locator.clone(),
    sender: tx.clone(),
  }).unwrap();

  drop(tx);

  let traversed = Arc::new(Mutex::new(HashSet::new()));
  let mut tasks = vec![];

  let transform_opts_base = Arc::new(OnTransformOpts {
    swc: OnTransformSwcOpts {
      use_esfuse_runtime: true,
      promisify_body: false,
    },
    user_data: args.opts.user_data.clone(),
  });

  let resolve_opts_base = Arc::new(OnResolveOpts {
    force_params: vec![StringKeyValue {
      name: String::from("transform"),
      value: String::from("js"),
    }],
    user_data: args.opts.user_data.clone(),
  });

  let mut is_first_store = true;

  while let Some(msg) = rx.recv().await {
    if !traversed.lock().unwrap().insert(msg.locator.clone()) {
      continue;
    }

    let is_first_iter = is_first_store;
    is_first_store = false;

    let build_results_accessor
      = build_results_container.clone();

    let project
      = project_base.clone();
    let bundle_opts
      = args.opts.clone();
    let transform_opts
      = transform_opts_base.clone();
    let resolve_opts
      = resolve_opts_base.clone();

    let task = tokio::spawn(async move {
      let mut transform_opts_iter
        = transform_opts.as_ref().clone();

      if is_first_iter && args.opts.promisify_entry_point {
        transform_opts_iter.swc.promisify_body = true;
      }
  
      let transform_result = super::transform::transform(&project, OnTransformArgs {
        locator: msg.locator.clone(),
        opts: transform_opts_iter,
      }).await;

      if let Err(transform_err) = transform_result.result {
        let mut build_results
          = build_results_accessor.lock().unwrap();

        build_results.insert(msg.locator.url, Err(transform_err));
        return;
      }

      let transform = transform_result.result.unwrap();

      let mut resolutions = HashMap::new();
      let mut resolution_errors = Vec::new();
  
      for import in &transform.imports {
        let mut resolution = super::resolve::resolve(&project, OnResolveArgs {
          request: import.specifier.clone(),
          issuer: Some(msg.locator.clone()),
          span: Some(import.span.clone()),
          opts: resolve_opts.as_ref().clone(),
        }).await;

        match &mut resolution.result {
          Ok(target_locator) => {
            let mut resolution_target_url = match target_locator.locator.kind {
              ModuleLocatorKind::External => None,
              _ => Some(target_locator.locator.url.clone()),
            };

            if !bundle_opts.traverse_vendors {
              if let Some(resolution_target_url_val) = &resolution_target_url {
                if import.specifier.as_str() != "esfuse/context" && (resolution_target_url_val.contains("/node_modules/") || resolution_target_url_val.contains("/packages/esfuse/sources/")) {
                  resolution_target_url = None;
                }
              }
            }

            if resolution_target_url.is_some() && !args.opts.only_entry_point {
              msg.sender.send(BundleMessage {
                locator: target_locator.locator.clone(),
                sender: msg.sender.clone(),
              }).unwrap();
            }

            resolutions
              .insert(import.specifier.clone(), resolution_target_url);
          }
  
          Err(err) => {
            resolution_errors.append(&mut err.diagnostics);
          }
        }
      }

      if !resolution_errors.is_empty() {
        build_results_accessor.lock().unwrap().insert(msg.locator.url.clone(), Err(
          CompilationError {diagnostics: resolution_errors}
        ));

        return;
      }

      if transform.mime_type != "text/javascript" && !is_first_iter {
        build_results_accessor.lock().unwrap().insert(msg.locator.url.clone(), Err(
          CompilationError::from_string(format!("Bundled modules can only be of type text/javascript; module {} seems to be {} instead", &msg.locator.url, &transform.mime_type))
        ));

        return;
      }

      // Note: We do this before the lock(), to ensure
      // we're staying locked as little as possible
      let bundle_module =
        BundleModule::new(msg.locator.clone(), transform, resolutions);

      build_results_accessor.lock().unwrap().insert(msg.locator.url, Ok(
        bundle_module,
      ));
    });

    if is_first_iter {
      task.await.unwrap();

      let build_results
        = build_results_container.lock().unwrap();

      if let Some(Ok(entry_module)) = build_results.values().next() {
        if entry_module.mime_type != "text/javascript" {
          return OnBundleResult {
            result: Ok(OnBundleResultData {
              entry: args.locator.url,
              mime_type: entry_module.mime_type.clone(),
              code: entry_module.code.clone(),
              map: String::new(),
            }),
            dependencies: vec![],
          }
        }
      }
    } else {
      tasks.push(task);
    }
  }

  // Wait for all tasks to finish
  for task in tasks {
    task.await.unwrap();
  }

  let mut build_results
    = build_results_container.lock().unwrap();

  let mut sorted_results: Vec<(String, Result<BundleModule, CompilationError>)>
    = build_results.drain().collect();

  sorted_results.sort_by(|a, b| {
    b.0.cmp(&a.0)
  });

  let mut meta = HashMap::new();

  for (url, module) in sorted_results {
    if final_nl_count > 0 {
      final_nl_count += 1;
      final_source += "\n";
    }
  
    match module {
      Ok(module) => {
        if let Some(mut map) = module.map {
          final_source_map.add_sourcemap(&mut map, final_nl_count as i64)
            .expect("Assertion failed: Expected the SWC-generated sourcemap to be well-structured");  
        }

        final_nl_count += module.newlines;
        final_source += module.code.as_str();

        meta.insert(url, BundleMeta {
          error: None,
          path: module.locator.physical_path(project).map(|b| b.to_string_lossy().to_string()),
          resolutions: module.resolutions,
        });
      },

      Err(err) => {
        meta.insert(url, BundleMeta {
          error: Some(err),
          path: None,
          resolutions: Default::default(),
        });
      },
    };
  }

  if final_nl_count > 0 {
    final_source += "\n";
  }
  
  final_source += format!("$esfuse$.meta({});\n", utils::serialize_json(&meta, &args.locator.url).unwrap()).as_str();

  if args.opts.require_on_load {
    final_source += format!("\n(typeof module !== 'undefined' ? module : {{}}).exports = $esfuse$.require({});\n", utils::serialize_json(&args.locator.url, &args.locator.url).unwrap()).as_str();
  }

  final_source += format!("\n//# sourceMappingURL={}\n", ModuleLocator::new(
    args.locator.kind,
    format!("{}.map", &args.locator.specifier),
    args.locator.params,
  ).url).as_str();

  OnBundleResult {
    result: Ok(OnBundleResultData {
      entry: args.locator.url,
      mime_type: String::from("text/javascript"),

      code: final_source,
      map: final_source_map.to_json(None).expect("Should have been able to serialize the source map"),
    }),
    dependencies: vec![
    ],
  }
}

fn count_newlines(s: &str) -> usize {
  s.as_bytes().iter().filter(|&&c| c == b'\n').count()
}
