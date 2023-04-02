use crate::types::*;
use crate::{CompilationError, Project};

use super::OnTransformResult;

pub fn transform_css(_project: &Project, module_source: OnFetchResult, _args: OnTransformArgs) -> Result<OnTransformResult, CompilationError> {
  Ok(OnTransformResult {
    mime_type: "text/css".to_string(),

    code: module_source.source.clone(),
    map: None,

    imports: vec![],
  })
}
