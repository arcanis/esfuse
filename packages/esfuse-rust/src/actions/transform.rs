use crate::{CompilationError, Project};
use crate::types::*;
use crate::utils;

use super::fetch::{fetch_no_hooks, fetch};

pub fn transform_no_hooks(project: &Project, args: OnTransformArgs) -> Result<OnTransformResult, CompilationError> {
  let fetch_output = fetch_no_hooks(project, OnFetchArgs { locator: args.locator.clone() })
    .map_err(|err| utils::errors::CompilationError::from_string(err.to_string()))?;

  crate::transforms::transform(project, fetch_output, args)
}

pub async fn transform(project: &Project, args: OnTransformArgs) -> Result<OnTransformResult, CompilationError> {
  let fetch_output = fetch(project, OnFetchArgs { locator: args.locator.clone() }).await
    .map_err(|err| utils::errors::CompilationError::from_string(err.to_string()))?;

  crate::transforms::transform(project, fetch_output, args)
}
