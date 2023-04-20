use base64::Engine;
use pnp::fs::ZipCache;

use crate::CompilationError;
use crate::Project;

use crate::types::*;
use crate::utils;

pub async fn fetch(project: &Project, args: OnFetchArgs) -> OnFetchResult {
  if args.locator.kind == ModuleLocatorKind::External {
    return OnFetchResult {
      result: Err(CompilationError::from_string(format!("Cannot fetch this module ({} is external)", args.locator.url))),
      dependencies: vec![],
    };
  }

  Project::resolve_plugin_hook(
    &project.on_fetch,
    &args.locator.url,
    args.clone(),
  ).await.unwrap_or_else(|| {
    fetch_no_hooks(project, args)
  })
}

pub fn fetch_no_hooks(project: &Project, args: OnFetchArgs) -> OnFetchResult {
  let ext = utils::get_extension(&args.locator.specifier);

  let transform_opt
    = args.locator.params.iter().rev().find(|p| p.name == "transform");

  if let Some(transform) = transform_opt {
    match transform.value.as_str() {
      "url" => {
        let stringified_url
          = serde_json::to_string(&args.locator.without_query().url).unwrap();

        return OnFetchResult {
          result: Ok(OnFetchResultData {
            locator: args.locator,
            mime_type: String::from("text/javascript"),
            source: format!("export default {};", stringified_url),
          }),
          dependencies: vec![],
        };
      },

      _ => {},
    }
  }

  match args.locator.physical_path(project) {
    Some(p) => {
      let source_res = pnp::fs::vpath(p.as_ref()).and_then(|res| match &res {
        pnp::fs::VPath::Native(p)
          => std::fs::read(p),
        pnp::fs::VPath::Virtual(info @ pnp::fs::VPathInfo { zip_path: None, .. })
          => std::fs::read(info.physical_base_path()),
        pnp::fs::VPath::Virtual(info @ pnp::fs::VPathInfo { zip_path: Some(zip_path), .. })
          => project.zip_cache.read(&info.physical_base_path(), zip_path),
      });

      match source_res {
        Ok(source_bytes) => {
          let mime_type
            = utils::get_mime_from_ext(&ext).to_string();

          let source = match utils::is_binary_mime_type(&mime_type) {
            true => base64::engine::general_purpose::STANDARD_NO_PAD.encode(&source_bytes),
            false => String::from_utf8_lossy(&source_bytes).into_owned(),
          };

          OnFetchResult {
            result: Ok(OnFetchResultData { locator: args.locator, mime_type, source }),
            dependencies: vec![],
          }
        }

        Err(_) => {
          OnFetchResult {
            result: Err(CompilationError::from_string(format!("Cannot fetch this module (an error happened while reading {})", args.locator.url))),
            dependencies: vec![],
          }    
        }
      }
    }

    None => {
      OnFetchResult {
        result: Err(CompilationError::from_string(format!("Cannot fetch this module (no fetcher configured for {})", args.locator.url))),
        dependencies: vec![].into(),
      }
    }
  }
}
