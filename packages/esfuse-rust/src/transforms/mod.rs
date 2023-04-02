use crate::{CompilationError, Project};
use crate::types::*;
use crate::utils;

mod css;
mod mdx;
mod swc;

pub use self::swc::OnTransformSwcOpts;

pub fn transform(project: &Project, module_source: OnFetchResult, args: OnTransformArgs) -> Result<OnTransformResult, CompilationError> {
  let ext = utils::get_extension(&module_source.locator.pathname);

  match ext.as_str() {
    ".css" => {
      self::css::transform_css(project, module_source, args)
    }

    ".jsx" | ".js" | ".ts" | ".tsx" => {
      self::swc::transform_swc(project, module_source, args)
    }

    ".mdx" => {
      if module_source.locator.params.iter().any(|pair| pair.name == "meta") {
        self::mdx::transform_mdx_meta(project, module_source, args)
      } else {
        self::mdx::transform_mdx(project, module_source, args)
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
