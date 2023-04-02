use pnp::fs::ZipCache;

use crate::CompilationError;
use crate::Project;

use crate::types::*;

pub async fn fetch(project: &Project, args: OnFetchArgs) -> Result<OnFetchResult, CompilationError> {
  if let Some(hook) = Project::resolve_plugin_hook(&project.on_fetch, &args.locator.url()) {
    return (hook.cb)(hook.data.clone(), args).await;
  } else {
    return fetch_no_hooks(project, args);
  }
}

pub fn fetch_no_hooks(project: &Project, args: OnFetchArgs) -> Result<OnFetchResult, CompilationError> {
  let p = args.locator.physical_path(project)
    .ok_or(CompilationError::from_str("Cannot fetch the module sources from this locator"))?;

  let source = pnp::fs::vpath(p.as_ref()).and_then(|res| match &res {
    pnp::fs::VPath::Native(p)
      => std::fs::read_to_string(p),
    pnp::fs::VPath::Virtual(info @ pnp::fs::VPathInfo { zip_path: None, .. })
      => std::fs::read_to_string(&info.physical_base_path()),
    pnp::fs::VPath::Virtual(info @ pnp::fs::VPathInfo { zip_path: Some(zip_path), .. })
      => project.zip_cache.read_to_string(&info.physical_base_path(), &zip_path),
  })
    .map_err(|_| CompilationError::from_str("Cannot fetch the module sources from this locator"))?;

  Ok(OnFetchResult {
    locator: args.locator.clone(),
    source,
  })
}
