extern crate queues;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

use crate::utils::GetLocatorVirtualPathOpts;
use crate::{CompilationError, utils};
use crate::Project;
use crate::transforms::OnTransformSwcOpts;
use crate::types::*;
use crate::utils::errors::Diagnostic;

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
      use_esfuse_runtime: args.opts.use_esfuse_runtime,
      promisify_body: false,
    },
    user_data: args.opts.user_data.clone(),
  });

  let resolve_opts_base = Arc::new(OnResolveOpts {
    force_params: vec![],
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
    let generated_module_folder
      = args.opts.generated_module_folder.clone();

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

        build_results.insert(current_locator.url.clone(), OnBatchModuleResult {
          locator: current_locator,
          result: Err(transform_err),
          dependencies: vec![],
        });

        return;
      }

      let transform = transform_result.result.unwrap();

      let (
        resolutions,
        resolution_errors,
      ) = resolve_all(&project, current_locator.clone(), &transform.imports, &bundle_opts, &resolve_opts).await;
  
      for resolution in resolutions.values() {
        if let Some(resolution_locator) = resolution {
          if args.opts.traverse_dependencies {
            msg.sender.send(BatchMessage {
              locator: resolution_locator.clone(),
              sender: msg.sender.clone(),
            }).unwrap();
          }
        }
      }

      if !resolution_errors.is_empty() {
        build_results_accessor.lock().unwrap().insert(current_locator.url.clone(), OnBatchModuleResult {
          locator: current_locator,
          result: Err(CompilationError {diagnostics: resolution_errors}),
          dependencies: vec![],
        });

        return;
      }

      // Note: We do this before the lock(), to ensure
      // we're staying locked as little as possible
      let mut batch_module =
        OnBatchModule::new(transform, resolutions);

      batch_module.imaginary_path = utils::get_locator_virtual_path(&project, &current_locator, GetLocatorVirtualPathOpts {
        mime_type: &batch_module.mime_type,
        generated_module_folder: &generated_module_folder,
      });
  
      build_results_accessor.lock().unwrap().insert(current_locator.url.clone(), OnBatchModuleResult {
        locator: current_locator,
        result: Ok(batch_module),
        dependencies: vec![],
      });
    });

    tasks.push(task);
  }

  // Wait for all tasks to finish
  for task in tasks {
    task.await.unwrap();
  }

  let mut results
    = Arc::try_unwrap(build_results_container).unwrap().into_inner().unwrap();

  let all_resolution_mappings
    = compute_resolution_mappings(&results);

  for (_, result) in results.iter_mut() {
    if let Ok(batch_module) = &mut result.result {
      let resolution_mappings
        = all_resolution_mappings
          .get(&result.locator.url)
          .unwrap();

      if batch_module.mime_type == "text/javascript" {
        let source_map
          = batch_module.map.as_mut().unwrap().to_json(None).unwrap();

        let dependency_postprocessing = utils::swc::persist_resolutions(
          &result.locator,
          &batch_module.code,
          &source_map,
          resolution_mappings,
        );

        if let Ok(dependency_postprocessing) = dependency_postprocessing {
          batch_module.set_code(dependency_postprocessing.code);
          batch_module.set_map(dependency_postprocessing.map);

          println!("Persisted resolutions for {}", &result.locator.url);
        }
      }
    }
  }

  OnBatchResult {
    results,
  }
}

async fn resolve_all(project: &Project, locator: ModuleLocator, imports: &[Import], bundle_opts: &OnBatchOpts, resolve_opts: &OnResolveOpts) -> (HashMap<String, Option<ModuleLocator>>, Vec<Diagnostic>) {
  let mut resolutions = HashMap::new();
  let mut resolution_errors = Vec::new();

  let issuer_dir_maybe
    = project.package_dir_from_locator(&locator);

  for import in imports {
    let mut resolution = super::resolve::resolve(&project, OnResolveArgs {
      kind: import.kind,
      request: import.specifier.clone(),
      issuer: Some(locator.clone()),
      span: Some(import.span.clone()),
      opts: resolve_opts.clone(),
    }).await;

    match &mut resolution.result {
      Ok(resolution_entry) => {
        let mut resolution_locator = match resolution_entry.locator.kind {
          ModuleLocatorKind::External => None,
          _ => Some(resolution_entry.locator.clone()),
        };

        if !bundle_opts.traverse_natives && resolution_entry.locator.specifier.ends_with(".node") {
          resolution_locator = None;
        }

        if !bundle_opts.traverse_vendors && resolution_entry.locator.url.contains("/node_modules/") {
          resolution_locator = None;
        }

        if !bundle_opts.traverse_packages {
          if let Some(package_dir) = project.package_dir_from_locator(&resolution_entry.locator) {
            if let Some(issuer_dir) = &issuer_dir_maybe {
              if package_dir != *issuer_dir {
                resolution_locator = None;
              }
            }
          }
        }

        //println!("{:?} | {:?} | {:?} | {:?}", &locator, &resolution_entry.locator, &issuer_dir, &resolution_dir);
        resolutions
          .insert(import.specifier.clone(), resolution_locator);
      }

      Err(err) => {
        if !import.optional {
          resolution_errors.append(&mut err.diagnostics);
        }
      }
    }
  }

  (resolutions, resolution_errors)
}

fn compute_resolution_mappings(modules: &HashMap<String, OnBatchModuleResult>) -> HashMap<String, HashMap<String, String>> {
  let mut all_module_mappings = HashMap::new();

  for (key, batch_module_result) in modules {
    let mut module_mappings = HashMap::new();

    if let Ok(iter_module) = &batch_module_result.result {
      let iter_imaginary_path = if let Some(iter_imaginary_path) = &iter_module.imaginary_path {
        iter_imaginary_path
      } else {
        continue;
      };

      for (request, resolution) in &iter_module.resolutions {
        let dependency_locator = if let Some(dependency_locator) = resolution {
          dependency_locator
        } else {
          continue;
        };

        let dependency_module_result = if let Some(dependency_module_result) = modules.get(&dependency_locator.url) {
          dependency_module_result
        } else {
          continue;
        };

        let dependency_module = if let Ok(dependency_module) = &dependency_module_result.result {
          dependency_module
        } else {
          continue;
        };

        let dependency_imaginary_path = if let Some(dependency_imaginary_path) = &dependency_module.imaginary_path {
          dependency_imaginary_path
        } else {
          continue;
        };

        module_mappings.insert(request.clone(), iter_imaginary_path.relative_to(dependency_imaginary_path).to_string());
      }
    }

    all_module_mappings.insert(key.clone(), module_mappings);
  }
  
  all_module_mappings
}
