use path_slash::PathBufExt;
use pathdiff::diff_paths;
use std::path::Path;
use std::collections::HashMap;
use std::cell::RefCell;
use std::path::PathBuf;

use crate::types::*;
use crate::utils;

#[derive(Default)]
pub struct Project {
  pub root: PathBuf,

  pub(crate) zip_cache: RefCell<pnp::fs::LruZipCache>,

  pub(crate) ns_to_path: HashMap<String, PathBuf>,
  pub(crate) path_to_ns: arca::path::Trie<String>,
}

impl Project {
  pub fn new<P: AsRef<Path>>(root: P) -> Self {
    let mut project = Self {
      root: root.as_ref().to_path_buf(),
      ..Default::default()
    };

    project.register_ns("app", root.as_ref());

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

  pub fn locator_from_path<P: AsRef<Path>>(&self, p: &P, params: &utils::QueryString) -> ModuleLocator {
    let base = self.path_to_ns.get_ancestor_record(p).unwrap();
    let p_rel = diff_paths(p, base.1).unwrap();

    let pathname = clean_path::clean(p_rel)
      .to_slash_lossy()
      .to_string();

    ModuleLocator::Path(ModuleLocatorData {
      pathname: format!("{}/{}", base.2, pathname),
      params: params.clone()
    })
  }
}
