#![deny(clippy::all)]

use std::sync::Arc;
use std::{path::PathBuf, collections::HashMap};

use esfuse::actions::bundle::BundleOutput;
use esfuse::actions::transform::{OnTransformArgs, OnTransformResult};
use esfuse::actions::resolve::{Resolution, OnResolveArgs};
use esfuse::CompilationError;
use napi::Either;
use napi::bindgen_prelude::*;
use napi_derive::napi;

extern crate napi_derive;

#[napi(object)]
pub struct ProjectDefinition {
  pub root: String,
  pub namespaces: HashMap<String, String>,
}

pub fn use_project(definition: &ProjectDefinition) -> esfuse::Project {
  let root = PathBuf::from(&definition.root);

  let mut project = esfuse::Project::new(&root);
  for (ns, path_string) in &definition.namespaces {
    project.register_ns(ns, &PathBuf::from(path_string));
  }

  project
}

#[napi(object)]
pub struct GetPathFromUrlRequest {
  pub project: ProjectDefinition,
  pub url: String,
}

#[napi]
pub fn get_path_from_url(req: GetPathFromUrlRequest) -> Option<String> {
  let project = use_project(&req.project);

  esfuse::types::ModuleLocator::from_url(req.url)
    .and_then(|locator| locator.physical_path(&project))
    .map(|path| path.to_string_lossy().to_string())
}

#[napi(object)]
pub struct GetUrlFromPathRequest {
  pub project: ProjectDefinition,
  pub path: String,
}

#[napi]
pub fn get_url_from_path(req: GetUrlFromPathRequest) -> String {
  let project = use_project(&req.project);

  project.locator_from_path(&PathBuf::from(req.path), &vec![].into()).url()
}

#[napi(object)]
pub struct ResolveRequest {
  pub project: ProjectDefinition,
  pub specifier: String,
  pub from: Option<String>,
  pub opts: Option<OnResolveArgs>,
}

#[napi]
pub async fn resolve_to_path(req: ResolveRequest) -> String {
  let project = use_project(&req.project);

  let from = req.from
    .map(PathBuf::from)
    .map(|p| project.locator_from_path(&p, &Default::default()));

  let resolve_opts = req.opts.unwrap_or_default();

  let res = esfuse::actions::resolve::resolve(
    &project,
    &req.specifier,
    from.as_ref(),
    Default::default(),
    &resolve_opts,
  ).await;

  if let Ok(Resolution::Module(locator)) = &res.result {
    if let Some(physical_path) = locator.physical_path(&project) {
      return physical_path.to_string_lossy().to_string();
    }
  }

  Default::default()
}

#[napi]
pub async fn resolve_to_url(req: ResolveRequest) -> String {
  let project = use_project(&req.project);

  let from = req.from
    .map(PathBuf::from)
    .map(|p| project.locator_from_path(&p, &Default::default()));

  let resolve_opts = req.opts.unwrap_or_default();

  let res = esfuse::actions::resolve::resolve(
    &project,
    &req.specifier,
    from.as_ref(),
    Default::default(),
    &resolve_opts,
  ).await;

  if let Ok(Resolution::Module(locator)) = &res.result {
    return locator.url();
  }

  Default::default()
}

#[napi(object)]
pub struct TransformRequest {
  pub project: ProjectDefinition,
  pub file: String,
  pub opts: Option<OnTransformArgs>,
}

#[napi(object)]
pub struct TransformOk {
  pub value: OnTransformResult,
  pub error: Null,
}

#[napi(object)]
pub struct TransformErr {
  pub value: Null,
  pub error: CompilationError,
}

#[napi]
pub fn transform_no_hooks(req: TransformRequest) -> Either<TransformOk, TransformErr> {
  let transform_opts = req.opts.unwrap_or_default();

  let project = use_project(&req.project);

  let locator
    = project.locator(req.file.as_str())
      .unwrap();

  let res = esfuse::actions::transform::transform_no_hooks(
    &project,
    &locator,
    &transform_opts,
  );

  match res {
    Ok(output) => {
      Either::A(TransformOk {
        value: output,
        error: Null,
      })
    },
    Err(err) => {
      Either::B(TransformErr {
        value: Null,
        error: err,
      })
    }
  }
}

#[napi]
pub async fn transform(req: TransformRequest) -> Either<TransformOk, TransformErr> {
  let transform_opts = req.opts.unwrap_or_default();

  let project = use_project(&req.project);

  let locator
    = project.locator(req.file.as_str())
      .unwrap();

  let res = esfuse::actions::transform::transform(
    &project,
    &locator,
    &transform_opts,
  ).await;

  match res {
    Ok(output) => {
      Either::A(TransformOk {
        value: output,
        error: Null,
      })
    },
    Err(err) => {
      Either::B(TransformErr {
        value: Null,
        error: err,
      })
    }
  }
}

#[napi(object)]
pub struct BundleRequest {
  pub project: ProjectDefinition,
  pub entry: String,
}

#[napi]
pub async fn bundle(req: BundleRequest) -> BundleOutput {
  let project = use_project(&req.project);

  let locator
    = project.locator(&req.entry)
      .unwrap();

  esfuse::actions::bundle::bundle(
    Arc::new(project),
    &locator,
  ).await
}
