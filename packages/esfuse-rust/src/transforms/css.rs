use crate::actions::fetch::OnFetchResult;
use crate::{CompilationError, Project};

use super::{OnTransformArgs, OnTransformResult};

pub fn transform_css(module_source: &OnFetchResult, _project: &Project, _opts: &OnTransformArgs) -> Result<OnTransformResult, CompilationError> {
  Ok(OnTransformResult {
    mime_type: "text/css".to_string(),

    code: module_source.source.clone(),
    map: None,

    imports: vec![],
  })
}
