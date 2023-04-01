use crate::actions::fetch::OnFetchResult;
use crate::{CompilationError, Project};
use crate::types::*;
use crate::utils;

mod css;
mod mdx;
mod swc;

pub use self::swc::OnTransformSwcArgs;

#[derive(Default)]
#[napi(object)]
pub struct OnTransformArgs {
  pub swc: OnTransformSwcArgs,
}

#[napi(object)]
pub struct ExtractedImport {
  pub specifier: String,
  pub span: Span,
}

#[napi(object)]
pub struct OnTransformResult {
  pub mime_type: String,

  pub code: String,
  pub map: Option<String>,

  pub imports: Vec<ExtractedImport>,
}

pub fn transform(module_source: &OnFetchResult, project: &Project, opts: &OnTransformArgs) -> Result<OnTransformResult, CompilationError> {
  let ext = utils::get_extension(&module_source.locator.pathname);

  match ext.as_str() {
    ".css" => {
      self::css::transform_css(module_source, project, opts)
    }

    ".jsx" | ".js" | ".ts" | ".tsx" => {
      self::swc::transform_swc(module_source, project, opts)
    }

    ".mdx" => {
      if module_source.locator.params.iter().any(|pair| pair.name == "meta") {
        self::mdx::transform_mdx_meta(module_source, project, opts)
      } else {
        self::mdx::transform_mdx(module_source, project, opts)
      }
    }

    _ => {
      Ok(OnTransformResult {
        mime_type: mime_guess::from_ext(&ext[1..])
          .first()
          .map(|m| m.to_string())
          .unwrap_or(String::from("text/plain")),

        code: module_source.source.clone(),
        map: None,

        imports: vec![],
      })
    }
  }
}
