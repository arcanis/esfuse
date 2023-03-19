use serde::Serialize;

use crate::{types::*, utils};

mod mdx;
mod swc;

pub struct TransformOutput {
  pub code: String,
  pub map: Option<String>,
  pub imports: Vec<String>,
}

#[derive(thiserror::Error, Debug, Serialize)]
#[serde(tag = "type")]
pub enum TransformError {
  #[error("Invalid module url: {url:?}")]
  InvalidUrl {
    url: String,
  },

  #[error("Module resolution failed")]
  ResolutionError {
    error: parcel_resolver::ResolverError,
  },

  #[error(transparent)]
  CompilationError(
    utils::errors::CompilationError,
  ),
}

pub fn transform(module_source: &ModuleBody, project: &Project) -> Result<TransformOutput, TransformError> {
  match utils::get_extension(&module_source.locator.pathname).as_str() {
    ".jsx" | ".js" | ".ts" | ".tsx" => {
      self::swc::transform_swc(module_source, project)
    }

    ".mdx" => {
      if module_source.locator.params.iter().any(|(k, _)| k == "meta") {
        self::mdx::transform_mdx_meta(module_source, project)
      } else {
        self::mdx::transform_mdx(module_source, project)
      }
    }

    _ => {
      Ok(TransformOutput {
        code: module_source.source.clone(),
        map: None,
        imports: vec![],
      })
    }
  }
}
