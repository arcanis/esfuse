#![deny(clippy::all)]

use std::str::FromStr;
use std::sync::Arc;
use std::{path::PathBuf, collections::HashMap};

use esfuse::types::ModuleLocator;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ErrorStrategy};
use napi_derive::napi;

extern crate napi_derive;

#[napi(object)]
pub struct ProjectHook {
  pub regexp: String,
  pub cb: JsFunction,
}

#[napi(object)]
pub struct ProjectDefinition {
  pub root: String,
  pub namespaces: HashMap<String, String>,

  pub on_resolve: Vec<ProjectHook>,
  pub on_fetch: Vec<ProjectHook>,
}

#[derive(Clone)]
struct HookData<T: 'static> {
  cb: ThreadsafeFunction<T, ErrorStrategy::Fatal>,
}

#[napi]
pub struct ProjectHandle {
  project: Arc<esfuse::Project>,
}

#[napi(object)]
pub struct GetFromLocatorRequest {
  pub locator: ModuleLocator,
}

#[napi(object)]
pub struct GetFromPathRequest {
  pub path: String,
}

#[napi(object)]
pub struct GetFromUrlRequest {
  pub url: String,
}

#[napi]
impl ProjectHandle {
  #[napi(factory)]
  pub fn create(definition: ProjectDefinition) -> Self {
    let project = Arc::new(
      use_project(definition),
    );

    Self {
      project,
    }
  }

  #[napi]
  pub fn dispose(&mut self) {
    let project = Arc::get_mut(&mut self.project).unwrap();

    project.on_resolve.clear();
    project.on_fetch.clear();
  }

  #[napi]
  pub fn get_path_from_locator(&self, req: GetFromLocatorRequest) -> Option<String> {
    req.locator.physical_path(&self.project)
      .map(|path| path.to_string_lossy().to_string())
  }

  #[napi]
  pub fn get_locator_from_path(&self, req: GetFromPathRequest) -> Option<esfuse::types::ModuleLocator> {
    self.project.locator_from_path(&PathBuf::from(req.path), &vec![])
  }

  #[napi]
  pub fn get_locator_from_url(&self, req: GetFromUrlRequest) -> Option<esfuse::types::ModuleLocator> {
    esfuse::types::ModuleLocator::from_url(&req.url)
  }

  #[napi]
  pub async fn resolve(&self, args: esfuse::types::OnResolveArgs) -> ResolveResult {
    let res = esfuse::actions::resolve::resolve(
      &self.project,
      args,
    ).await;

    match res.result {
      Ok(value) => ResolveResult { value: Some(value), error: None, dependencies: res.dependencies },
      Err(error) => ResolveResult { value: None, error: Some(error), dependencies: res.dependencies },
    }
  }

  #[napi]
  pub fn transform_no_hooks(&self, args: esfuse::types::OnTransformArgs) -> TransformResult {
    let res = esfuse::actions::transform::transform_no_hooks(
      &self.project,
      args,
    );

    match res.result {
      Ok(value) => TransformResult { value: Some(value), error: None, dependencies: res.dependencies },
      Err(error) => TransformResult { value: None, error: Some(error), dependencies: res.dependencies },
    }
  }

  #[napi]
  pub async fn transform(&self, args: esfuse::types::OnTransformArgs) -> TransformResult {
    let res = esfuse::actions::transform::transform(
      &self.project,
      args,
    ).await;

    match res.result {
      Ok(value) => TransformResult { value: Some(value), error: None, dependencies: res.dependencies },
      Err(error) => TransformResult { value: None, error: Some(error), dependencies: res.dependencies },
    }
  }

  #[napi]
  pub async fn bundle(&self, args: esfuse::types::OnBundleArgs) -> BundleResult {
    let res = esfuse::actions::bundle::bundle(
      self.project.clone(),
      args,
    ).await;

    match res.result {
      Ok(value) => BundleResult { value: Some(value), error: None, dependencies: res.dependencies },
      Err(error) => BundleResult { value: None, error: Some(error), dependencies: res.dependencies },
    }
  }
}

pub fn use_project(definition: ProjectDefinition) -> esfuse::Project {
  let root = PathBuf::from(&definition.root);

  let mut project = esfuse::Project::new(&root);
  for (ns, path_string) in &definition.namespaces {
    project.register_ns(ns, &PathBuf::from(path_string));
  }

  for hook in definition.on_resolve {
    let tsfn: ThreadsafeFunction<esfuse::types::OnResolveArgs, ErrorStrategy::Fatal> = hook.cb
      .create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))
      .unwrap();

    project.on_resolve.push(esfuse::types::PluginHook {
      regexp: esfuse::utils::Regex::from_str(&hook.regexp).unwrap(),
      params: Default::default(),

      cb: |hook_data, args| {
        Box::pin(async move {
          let issuer = args.issuer.clone();
          let span = args.span.clone();

          let user
            = hook_data.downcast_ref::<HookData<esfuse::types::OnResolveArgs>>().unwrap().clone();
          let future
            = user.cb.call_async::<Promise<Option<ResolveResult>>>(args);

          match future.await {
            Ok(promise) => {
              match promise.await {
                Ok(hook_maybe) => hook_maybe.map(|hook_res| {
                  esfuse::types::OnResolveResult {
                    result: hook_res.value.ok_or_else(|| hook_res.error.unwrap()),
                    dependencies: vec![],
                  } 
                }),
  
                Err(err) => Some(esfuse::types::OnResolveResult {
                  result: Err(esfuse::CompilationError::from_string_with_highlight(err.to_string(), esfuse::utils::errors::Highlight {
                    source: issuer.map(|locator| locator.url),
                    subject: None,
                    label: None,
                    span: span.clone(),
                  })),
                  dependencies: vec![],
                }),
              }
            },

            Err(err) => Some(esfuse::types::OnResolveResult {
              result: Err(esfuse::CompilationError::from_string_with_highlight(err.to_string(), esfuse::utils::errors::Highlight {
                source: issuer.map(|locator| locator.url),
                subject: None,
                label: None,
                span: span.clone(),
              })),
              dependencies: vec![],
            }),
          }
        })
      },

      data: Arc::new(Box::new(HookData::<esfuse::types::OnResolveArgs> {
        cb: tsfn,
      }))
    });
  }

  for hook in definition.on_fetch {
    let tsfn: ThreadsafeFunction<esfuse::types::OnFetchArgs, ErrorStrategy::Fatal> = hook.cb
      .create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))
      .unwrap();

    project.on_fetch.push(esfuse::types::PluginHook {
      regexp: esfuse::utils::Regex::from_str(&hook.regexp).unwrap(),
      params: Default::default(),

      cb: |hook_data, args| {
        Box::pin(async move {
          let user
            = hook_data.downcast_ref::<HookData<esfuse::types::OnFetchArgs>>().unwrap().clone();
          let future
            = user.cb.call_async::<Promise<Option<FetchResult>>>(args);

          match future.await {
            Ok(promise) => {
              match promise.await {
                Ok(hook_maybe) => hook_maybe.map(|hook_res| {
                  esfuse::types::OnFetchResult {
                    result: hook_res.value.ok_or_else(|| hook_res.error.unwrap()),
                    dependencies: vec![],
                  } 
                }),
  
                Err(err) => Some(esfuse::types::OnFetchResult {
                  result: Err(esfuse::CompilationError::from_napi(err)),
                  dependencies: vec![],
                }),
              }
            },

            Err(err) => Some(esfuse::types::OnFetchResult {
              result: Err(esfuse::CompilationError::from_napi(err)),
              dependencies: vec![],
            }),
          }
        })
      },

      data: Arc::new(Box::new(HookData::<esfuse::types::OnFetchArgs> {
        cb: tsfn,
      }))
    });
  }

  project
}

#[napi(object)]
pub struct ResolveResult {
  pub value: Option<esfuse::types::OnResolveResultData>,
  pub error: Option<esfuse::CompilationError>,
  pub dependencies: Vec<esfuse::types::ModuleLocator>,
}

#[napi(object)]
pub struct FetchResult {
  pub value: Option<esfuse::types::OnFetchResultData>,
  pub error: Option<esfuse::CompilationError>,
  pub dependencies: Vec<esfuse::types::ModuleLocator>,
}

#[napi(object)]
pub struct TransformResult {
  pub value: Option<esfuse::types::OnTransformResultData>,
  pub error: Option<esfuse::CompilationError>,
  pub dependencies: Vec<esfuse::types::ModuleLocator>,
}

#[napi(object)]
pub struct BundleResult {
  pub value: Option<esfuse::types::OnBundleResultData>,
  pub error: Option<esfuse::CompilationError>,
  pub dependencies: Vec<esfuse::types::ModuleLocator>,
}
