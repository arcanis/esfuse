use crate::{CompilationError, Project};
use crate::types::*;
use crate::utils;

use super::fetch::{fetch_no_hooks, fetch};

pub use crate::transforms::{
  OnTransformArgs,
  OnTransformSwcArgs,
  OnTransformResult,
};

pub fn transform_no_hooks(project: &Project, locator: &ModuleLocator, opts: &OnTransformArgs) -> Result<OnTransformResult, CompilationError> {
  let fetch_output = fetch_no_hooks(project, locator)
    .map_err(|err| utils::errors::CompilationError::from_string(err.to_string()))?;

  crate::transforms::transform(&fetch_output, project, &opts)
}

pub async fn transform(project: &Project, locator: &ModuleLocator, opts: &OnTransformArgs) -> Result<OnTransformResult, CompilationError> {
  let fetch_output = fetch(project, locator).await
    .map_err(|err| utils::errors::CompilationError::from_string(err.to_string()))?;

  crate::transforms::transform(&fetch_output, project, &opts)
}
