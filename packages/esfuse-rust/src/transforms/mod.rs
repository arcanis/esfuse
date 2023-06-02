use crate::types::*;
use crate::Project;

mod css;
mod mdx;
mod swc;

pub use self::swc::OnTransformSwcOpts;

pub fn transform(project: &Project, fetch_data: OnFetchResultData, args: OnTransformArgs) -> OnTransformResult {
  match fetch_data.mime_type.as_str() {
    "text/css" => {
      if fetch_data.locator.params.iter().any(|pair| pair.name == "transform" && pair.value == "css") {
        self::css::transform_css(project, fetch_data, args)
      } else {
        self::css::transform_css_js(project, fetch_data, args)
      }
    }

    "text/javascript" => {
      self::swc::transform_swc(project, fetch_data, args)
    }

    "text/markdown" => {
      if fetch_data.locator.params.iter().any(|pair| pair.name == "meta") {
        self::mdx::transform_mdx_meta(project, fetch_data, args)
      } else {
        self::mdx::transform_mdx(project, fetch_data, args)
      }
    }

    _ => {
      OnTransformResult {
        result: Ok(OnTransformResultData {
          mime_type: fetch_data.mime_type,
          code: fetch_data.source,
          map: None,
          imports: vec![],
        }),
        dependencies: vec![],
      }
    }
  }
}
