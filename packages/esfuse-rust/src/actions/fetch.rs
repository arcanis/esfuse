use pnp::fs::ZipCache;

use crate::Project;

use crate::types::*;

#[derive(Clone)]
pub struct OnFetchResult {
  pub locator: ModuleLocator,
  pub source: String,
}

pub async fn fetch(project: &Project, locator: &ModuleLocator) -> Result<OnFetchResult, std::io::Error> {
  let hook
    = Project::resolve_plugin_hook(&project.on_fetch, &locator.url());

  if let Some(fetcher) = hook {
    return fetcher(project, locator).await;
  } else {
    return fetch_no_hooks(project, locator);
  }
}

pub fn fetch_no_hooks(project: &Project, locator: &ModuleLocator) -> Result<OnFetchResult, std::io::Error> {
  let source = match locator {
    ModuleLocator::Path(_) => {
      let p = locator.physical_path(project).unwrap();

      pnp::fs::vpath(p.as_ref()).and_then(|res| match &res {
        pnp::fs::VPath::Native(p)
          => std::fs::read_to_string(p),
        pnp::fs::VPath::Virtual(info @ pnp::fs::VPathInfo { zip_path: None, .. })
          => std::fs::read_to_string(&info.physical_base_path()),
        pnp::fs::VPath::Virtual(info @ pnp::fs::VPathInfo { zip_path: Some(zip_path), .. })
          => project.zip_cache.read_to_string(&info.physical_base_path(), &zip_path),
      })
    },

    _ => {
      return Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Cannot fetch the module sources from this locator",
      ));
    }
  }?;

  Ok(OnFetchResult {
    locator: locator.clone(),
    source,
  })
}
