use parcel_resolver::CacheCow;
use path_slash::PathBufExt;
use pathdiff::diff_paths;
use std::borrow::Cow;
use std::path::Path;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::types::*;
use crate::utils;

pub struct Project {
  pub root: Cow<'static, Path>,

  pub on_resolve: Vec<PluginHook<OnResolveArgs, OnResolveResult>>,
  pub on_fetch: Vec<PluginHook<OnFetchArgs, OnFetchResult>>,

  pub(crate) resolver: parcel_resolver::Resolver<'static, parcel_resolver::OsFileSystem>,
  pub(crate) zip_cache: pnp::fs::LruZipCache<Vec<u8>>,

  pub(crate) ns_to_path: HashMap<String, PathBuf>,
  pub(crate) path_to_ns: arca::path::Trie<String>,
}

impl Project {
  pub async fn resolve_plugin_hook<'a, TArgs : Clone, TRes>(hooks: &'a Vec<PluginHook<TArgs, TRes>>, str: &str, args: TArgs) -> Option<TRes> {
    for hook in hooks {
      if hook.regexp.is_match(str).unwrap() {
        if let Some(res) = (hook.cb)(hook.data.clone(), args.clone()).await {
          return Some(res)
        }
      }
    }

    None
  }

  pub fn new(root: &Path) -> Self {
    let resolver_fs = parcel_resolver::OsFileSystem::default();
    let resolver_cache = parcel_resolver::Cache::new(resolver_fs);

    let mut project = Self {
      root: Cow::Owned(root.to_path_buf()),

      on_resolve: Default::default(),
      on_fetch: Default::default(),

      resolver: parcel_resolver::Resolver::parcel(
        Cow::Owned(root.to_path_buf()),
        CacheCow::Owned(resolver_cache),
      ),

      zip_cache: pnp::fs::LruZipCache::new(50, pnp::fs::open_zip_via_read),

      ns_to_path: Default::default(),
      path_to_ns: Default::default(),
    };
  
    project.register_ns("app", root);

    project
  }

  pub fn register_ns<S: AsRef<str>, P: AsRef<Path>>(&mut self, ns: S, p: P) {
    self.ns_to_path.insert(
      ns.as_ref().to_string(),
      p.as_ref().to_path_buf(),
    );

    self.path_to_ns.insert(
      p.as_ref(),
      ns.as_ref().to_string(),
    );
  }

  pub fn root_ns<P: AsRef<str>>(&self, ns: P) -> &PathBuf {
    self.ns_to_path.get(ns.as_ref()).unwrap()
  }

  pub fn locator(&self, specifier: &str) -> Option<ModuleLocator> {
    if specifier.starts_with("/_dev/") || specifier.contains(':') {
      ModuleLocator::from_url(specifier)
    } else if specifier.starts_with('/') {
      let (pathname, query)
        = utils::split_query(specifier);

      let params
        = query.map_or(Default::default(), utils::parse_query);

      self.locator_from_path(&PathBuf::from(pathname), &params)
    } else {
      None
    }
  }

  pub fn locator_from_path(&self, p: &Path, params: &[StringKeyValue]) -> Option<ModuleLocator> {
    if let Some(base) = self.path_to_ns.get_ancestor_record(&p) {
      let p_rel = diff_paths(p, base.1).unwrap();

      let pathname = clean_path::clean(p_rel)
        .to_slash_lossy()
        .to_string();
  
      Some(ModuleLocator::new(
        ModuleLocatorKind::File,
        format!("{}/{}", base.2, pathname),
        params.to_vec(),
      ))
    } else {
      None
    }
  }
}
