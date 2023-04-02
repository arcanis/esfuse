extern crate queues;

use parcel_sourcemap::SourceMap;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

use crate::CompilationError;
use crate::Project;
use crate::transforms::OnTransformSwcOpts;
use crate::types::*;

use super::resolve::resolve;
use super::transform::transform;

#[derive(Debug)]
struct BundleMessage {
  locator: ModuleLocator,
  sender: UnboundedSender<BundleMessage>,
}

struct BundleModule {
  code: String,
  map: Option<SourceMap>,
  newlines: usize,
  resolutions: HashMap<String, String>,
}

pub async fn bundle(project_base: Arc<Project>, args: OnBundleArgs) -> Result<OnBundleResult, CompilationError> {
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
    },
  });

  let resolve_opts_base = Arc::new(OnResolveOpts {
    force_params: vec![StringKeyValue {
      name: String::from("transform"),
      value: Some(String::from("js")),
    }].into(),
  });

  while let Some(msg) = rx.recv().await {
    if !traversed.lock().unwrap().insert(msg.locator.clone()) {
      continue;
    }

    let build_results_accessor
      = build_results_container.clone();

    let project
      = project_base.clone();
    let transform_opts
      = transform_opts_base.clone();
    let resolve_opts
      = resolve_opts_base.clone();

    let task = tokio::spawn(async move {
      let transform_result = transform(&project, OnTransformArgs {
        locator: msg.locator.clone(),
        opts: transform_opts.as_ref().clone(),
      }).await;

      let transform_output = transform_result.unwrap();

      let source = &transform_output.code;
      let source_nl_count = count_newlines(source.as_str());

      let source_map = transform_output.map.map(|str| {
        parcel_sourcemap::SourceMap::from_json("/", &str)
          .expect("Assertion failed: Expected the SWC-generated sourcemap to be readable")
      });

      let mut resolutions = HashMap::new();
      let mut resolution_errors = Vec::new();
  
      for import in transform_output.imports {
        let mut resolution = resolve(&project, OnResolveArgs {
          request: import.specifier.clone(),
          issuer: Some(msg.locator.clone()),
          span: import.span,
          opts: resolve_opts.as_ref().clone(),
        }).await;

        match &mut resolution.result {
          Ok(target_locator) => {
            resolutions
              .insert(import.specifier, target_locator.url());
  
            msg.sender.send(BundleMessage {
              locator: target_locator.clone(),
              sender: msg.sender.clone(),
            }).unwrap();
          }
  
          Err(err) => {
            resolution_errors.append(&mut err.diagnostics);
          }
        }
      }

      let build_result = match resolution_errors.is_empty() {
        true => Ok(BundleModule {
          code: transform_output.code,
          map: source_map,
          newlines: source_nl_count,
          resolutions,
        }),

        false => Err(CompilationError {
          diagnostics: resolution_errors,
        }),
      };

      let mut build_results
        = build_results_accessor.lock().unwrap();

      build_results.insert(msg.locator.url(), build_result);
    });

    tasks.push(task);
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

  let mut final_nl_count = 0 as usize;
  let mut final_source = String::default();
  let mut final_source_map = parcel_sourcemap::SourceMap::new("");

  let mut errors = HashMap::new();
  let mut resolutions = HashMap::new();

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

        resolutions.insert(url, module.resolutions);
      },

      Err(err) => {
        errors.insert(url, err);
      },
    };
  }

  Ok(OnBundleResult {
    entry: args.locator.url(),
    mime_type: String::from("text/javascript"),

    code: final_source,
    map: final_source_map.to_json(None).expect("Should have been able to serialize the source map"),

    errors,
    resolutions,
  })
}

fn count_newlines(s: &str) -> usize {
  s.as_bytes().iter().filter(|&&c| c == b'\n').count()
}
