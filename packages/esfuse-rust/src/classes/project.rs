use arca::Path;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use parcel_resolver::CacheCow;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::types::*;
use crate::utils;

pub struct Project {
  pub root: Cow<'static, Path>,

  pub on_resolve: Vec<PluginHook<OnResolveArgs, OnResolveResult>>,
  pub on_fetch: Vec<PluginHook<OnFetchArgs, OnFetchResult>>,

  pub(crate) resolver: parcel_resolver::Resolver<'static, parcel_resolver::OsFileSystem>,
  pub(crate) zip_cache: pnp::fs::LruZipCache<Vec<u8>>,

  pub(crate) ns_to_path: HashMap<String, Path>,
  pub(crate) path_to_ns: arca::Trie<String>,

  pub(crate) package_json_finder: utils::FileFinder,
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
      root: Cow::Owned(root.clone()),

      on_resolve: Default::default(),
      on_fetch: Default::default(),

      resolver: parcel_resolver::Resolver::parcel(
        Cow::Owned(root.to_path_buf()),
        CacheCow::Owned(resolver_cache),
      ),

      zip_cache: pnp::fs::LruZipCache::new(50, pnp::fs::open_zip_via_read),

      ns_to_path: Default::default(),
      path_to_ns: Default::default(),

      package_json_finder: utils::FileFinder::new("package.json"),
    };
  
    project.register_ns("app", &root.clone());

    project
  }

  pub fn register_ns<S: AsRef<str>>(&mut self, ns: S, p: &Path) {
    self.ns_to_path.insert(
      ns.as_ref().to_string(),
      p.clone(),
    );

    self.path_to_ns.insert(
      p.clone(),
      ns.as_ref().to_string(),
    );
  }

  pub fn root_ns<P: AsRef<str>>(&self, ns: P) -> &Path {
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

      self.locator_from_path(&Path::from(pathname), &params)
    } else {
      None
    }
  }

  pub fn locator_from_path(&self, p: &Path, params: &[StringKeyValue]) -> Option<ModuleLocator> {
    self.ns_qualified_from_path(p).map(|specifier| {
      ModuleLocator::new(
        ModuleLocatorKind::File,
        specifier,
        params.to_vec(),
      )
    })
  }

  pub fn package_dir_from_locator(&self, locator: &ModuleLocator) -> Option<Path> {
    locator.physical_path(self).and_then(|path| {
      self.package_json_finder.find_file(&path)
    })
  }

  pub fn ns_qualified_from_path(&self, p: &Path) -> Option<String> {
    self.path_to_ns.get_ancestor_record(&p).map(|base| {
      let p_rel = p.relative_to(base.1);
      format!("{}/{}", base.2, p_rel.as_str())
    })
  }

  pub fn path_from_ns_qualified(&self, str: &str) -> Path {
    let (ns, pathname) = parse_file_pathname(&str);
    self.root_ns(ns).join(&pathname)
  }
}

fn parse_file_pathname(str: &str) -> (&str, Path) {
  lazy_static! {
    static ref RE: Regex = Regex::new(r"^([^/?]+)/(.*)$").unwrap();
  }

  let captures = RE.captures(str.as_ref()).unwrap().unwrap();

  let ns = captures.get(1).unwrap();
  let pathname = captures.get(2).unwrap();

  (ns.as_str(), Path::from(pathname.as_str()))
}
