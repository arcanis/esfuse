#![deny(clippy::all)]

use std::str::FromStr;
use std::sync::Arc;
use std::{path::PathBuf, collections::HashMap};

use esfuse::types::OnResolveArgs;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ErrorStrategy};
use napi_derive::napi;

extern crate napi_derive;

#[napi(object)]
pub struct ProjectHook {
  pub regex: String,
  pub cb: JsFunction,
}

#[napi(object)]
pub struct ProjectDefinition {
  pub root: String,
  pub namespaces: HashMap<String, String>,

  pub on_resolve: Vec<ProjectHook>,
}

#[derive(Clone)]
struct HookData {
  cb: ThreadsafeFunction<OnResolveArgs, ErrorStrategy::Fatal>,
}

#[napi]
pub struct ProjectHandle {
  project: Arc<esfuse::Project>,
}

#[napi(object)]
pub struct GetPathFromUrlRequest {
  pub url: String,
}

#[napi(object)]
pub struct GetUrlFromPathRequest {
  pub path: String,
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
  pub fn get_path_from_url(&self, req: GetPathFromUrlRequest) -> Option<String> {
    esfuse::types::ModuleLocator::from_url(req.url)
      .and_then(|locator| locator.physical_path(&self.project))
      .map(|path| path.to_string_lossy().to_string())
  }

  #[napi]
  pub fn get_url_from_path(&self, req: GetUrlFromPathRequest) -> String {
    self.project.locator_from_path(&PathBuf::from(req.path), &vec![]).url()
  }

  #[napi]
  pub async fn resolve(&self, args: OnResolveArgs) -> ResolveResult {
    let res = esfuse::actions::resolve::resolve(
      &self.project,
      args,
    ).await.result;

    match res {
      Ok(value) => ResolveResult { value: Some(value), error: None },
      Err(error) => ResolveResult { value: None, error: Some(error) },
    }
  }

  #[napi]
  pub fn transform_no_hooks(&self, args: esfuse::types::OnTransformArgs) -> TransformResult {
    let res = esfuse::actions::transform::transform_no_hooks(
      &self.project,
      args,
    );

    match res {
      Ok(value) => TransformResult { value: Some(value), error: None },
      Err(error) => TransformResult { value: None, error: Some(error) },
    }
  }

  #[napi]
  pub async fn transform(&self, args: esfuse::types::OnTransformArgs) -> TransformResult {
    let res = esfuse::actions::transform::transform(
      &self.project,
      args,
    ).await;

    match res {
      Ok(value) => TransformResult { value: Some(value), error: None },
      Err(error) => TransformResult { value: None, error: Some(error) },
    }
  }

  #[napi]
  pub async fn bundle(&self, args: esfuse::types::OnBundleArgs) -> BundleResult {
    let res = esfuse::actions::bundle::bundle(
      self.project.clone(),
      args,
    ).await;

    match res {
      Ok(value) => BundleResult { value: Some(value), error: None },
      Err(error) => BundleResult { value: None, error: Some(error) },
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
      regexp: esfuse::utils::Regex::from_str("fi").unwrap(),
      params: Default::default(),

      cb: |data, args| {
        Box::pin(async move {
          let span = args.span.clone();

          let user = data.downcast_ref::<HookData>().unwrap().clone();
          let future = user.cb.call_async::<esfuse::types::ModuleLocator>(args);
  
          let res = future.await;

          res.map_err(|_| esfuse::CompilationError::from_str_with_span("Resolution failed", span))
      })
      },

      data: Arc::new(Box::new(HookData {
        cb: tsfn,
      }))
    });
  }

  project
}

#[napi]
pub struct ResolveResult {
  pub value: Option<esfuse::types::ModuleLocator>,
  pub error: Option<esfuse::CompilationError>,
}

#[napi]
pub struct TransformResult {
  pub value: Option<esfuse::types::OnTransformResult>,
  pub error: Option<esfuse::CompilationError>,
}

#[napi]
pub struct BundleResult {
  pub value: Option<esfuse::types::OnBundleResult>,
  pub error: Option<esfuse::CompilationError>,
}
