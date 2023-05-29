extern crate queues;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

use crate::CompilationError;
use crate::Project;
use crate::transforms::OnTransformSwcOpts;
use crate::types::*;

#[derive(Debug)]
struct BatchMessage {
  locator: ModuleLocator,
  sender: UnboundedSender<BatchMessage>,
}

pub async fn batch(project_base: Arc<Project>, args: OnBatchArgs) -> OnBatchResult {
  let build_results_container
    = Arc::new(Mutex::new(HashMap::new()));

  let (tx, mut rx)
    = tokio::sync::mpsc::unbounded_channel();

  for locator in args.locators {
    tx.send(BatchMessage {
      locator: locator.clone(),
      sender: tx.clone(),
    }).unwrap();
  }

  drop(tx);

  let traversed = Arc::new(Mutex::new(HashSet::new()));
  let mut tasks = vec![];

  let transform_opts_base = Arc::new(OnTransformOpts {
    static_resolutions: Default::default(),
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
    let current_locator = msg.locator;

    let task = tokio::spawn(async move {
      let mut transform_opts_iter
        = transform_opts.as_ref().clone();

      if is_first_iter && args.opts.promisify_entry_point {
        transform_opts_iter.swc.promisify_body = true;
      }
  
      let transform_result = super::transform::transform(&project, OnTransformArgs {
        locator: current_locator.clone(),
        opts: transform_opts_iter,
      }).await;

      if let Err(transform_err) = transform_result.result {
        let mut build_results
          = build_results_accessor.lock().unwrap();

        build_results.insert(current_locator.url, OnBatchModuleResult {
          result: Err(transform_err),
          dependencies: vec![],
        });

        return;
      }

      let transform = transform_result.result.unwrap();

      let mut resolutions = HashMap::new();
      let mut resolution_errors = Vec::new();
  
      for import in &transform.imports {
        let mut resolution = super::resolve::resolve(&project, OnResolveArgs {
          kind: import.kind,
          request: import.specifier.clone(),
          issuer: Some(current_locator.clone()),
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

            if !bundle_opts.traverse_packages {
              if project.package_dir_from_locator(&current_locator) != project.package_dir_from_locator(&target_locator.locator) {
                resolution_target_url = None;
              }
            }

            if resolution_target_url.is_some() && args.opts.traverse_dependencies {
              msg.sender.send(BatchMessage {
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
        build_results_accessor.lock().unwrap().insert(current_locator.url.clone(), OnBatchModuleResult {
          result: Err(CompilationError {diagnostics: resolution_errors}),
          dependencies: vec![],
        });

        return;
      }

      if transform.mime_type != "text/javascript" && !is_first_iter {
        build_results_accessor.lock().unwrap().insert(current_locator.url.clone(), OnBatchModuleResult {
          result: Err(CompilationError::from_string(format!("Bundled modules can only be of type text/javascript; module {} seems to be {} instead", &current_locator.url, &transform.mime_type))),
          dependencies: vec![],
        });

        return;
      }

      // Note: We do this before the lock(), to ensure
      // we're staying locked as little as possible
      let bundle_module =
        OnBatchModule::new(current_locator.clone(), transform, resolutions);

      build_results_accessor.lock().unwrap().insert(current_locator.url, OnBatchModuleResult {
        result: Ok(bundle_module),
        dependencies: vec![],
      });
    });

    tasks.push(task);
  }

  // Wait for all tasks to finish
  for task in tasks {
    task.await.unwrap();
  }

  OnBatchResult {
    results: Arc::try_unwrap(build_results_container).unwrap().into_inner().unwrap(),
  }
}
