use fancy_regex::Regex;
use lazy_static::lazy_static;
use pnp::fs::ZipCache;
use serde::Serialize;
use std::path::PathBuf;

pub use crate::classes::*;
use crate::{utils, transforms::TransformError};

#[derive(Clone, Debug, Serialize)]
pub struct Highlight {
  pub label: Option<String>,
  pub span: Span,
}

#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
  pub message: String,
  pub highlights: Vec<Highlight>,
}

#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
pub struct Span {
  pub start: Position,
  pub end: Position,
}

#[derive(Debug, Serialize, Clone, Eq, PartialEq)]
pub struct Position {
  pub row: usize,
  pub col: usize,
}

impl Span {
  pub fn from_swc(span: &swc_common::Span, source_map: &swc_common::SourceMap) -> Self {
    if span.lo.is_dummy() || span.hi.is_dummy() {
      return Span {
        start: Position {
          row: 1,
          col: 1,
        },

        end: Position {
          row: 1,
          col: 1,
        },
      };
    }

    let start = source_map.lookup_char_pos(span.lo);
    let end = source_map.lookup_char_pos(span.hi);

    Span {
      start: Position {
        row: start.line,
        col: start.col_display + 1,
      },

      end: Position {
        row: end.line,
        col: end.col_display,
      },
    }
  }
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModuleLocatorData {
  pub pathname: String,
  pub params: Vec<(String, Option<String>)>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ModuleLocator {
  Path(ModuleLocatorData),
}

impl ModuleLocator {
  pub fn from_url<P: AsRef<str>>(url: P) -> Result<Self, TransformError> {
    lazy_static! {
      static ref RE: Regex = Regex::new(r"/_dev/([^/?]+)/([^?]*)(.*)$").unwrap();
    }

    if let Some(captures) = RE.captures(url.as_ref()).unwrap() {
      if let (Some(kind), Some(pathname), Some(qs)) = (captures.get(1), captures.get(2), captures.get(3)) {
        let data = ModuleLocatorData {
          pathname: String::from(pathname.as_str()),
          params: utils::parse_query(Some(String::from(qs.as_str()))),
        };

        return match kind.as_str() {
          "file" => Ok(ModuleLocator::Path(data)),

          _ => Err(TransformError::InvalidUrl {
            url: String::from(url.as_ref()),
          })
        };
      }
    }

    Err(TransformError::InvalidUrl {
      url: String::from(url.as_ref())
    })
  }

  pub fn without_query(&self) -> Self {
    let mut clone = self.clone();
    clone.params.clear();
    clone
  }

  pub fn url(&self) -> String {
    format!("/_dev/file/{}{}", &self.pathname, utils::stringify_query(&self.params))
  }

  pub fn physical_path(&self, project: &Project) -> Option<PathBuf> {
    match self {
      ModuleLocator::Path(data) => {
        let (ns, pathname) = parse_file_pathname(&data.pathname);
        Some(project.root_ns(ns).join(pathname))
      },
    }
  }

  pub fn fetch(&self, project: &Project) -> Result<ModuleBody, std::io::Error> {
    let source = match self {
      ModuleLocator::Path(_) => {
        let p = self.physical_path(project).unwrap();

        pnp::fs::vpath(p.as_ref()).map(|res| match res {
          pnp::fs::VPath::Native(p) => std::fs::read_to_string(p),
          pnp::fs::VPath::Zip(zip_path, sub) => project.zip_cache.borrow_mut().read_to_string(&zip_path, &sub),
        })?
      },
    }?;

    Ok(ModuleBody {
      locator: self.clone(),
      source,
    })
  }
}

impl std::ops::Deref for ModuleLocator {
  type Target = ModuleLocatorData;
  fn deref(&self) -> &ModuleLocatorData {
    match self {
      ModuleLocator::Path(data) => data,
    }
  }
}

impl std::ops::DerefMut for ModuleLocator {
  fn deref_mut(&mut self) -> &mut ModuleLocatorData {
    match self {
      ModuleLocator::Path(data) => data,
    }
  }
}

#[derive(Clone)]
pub struct ModuleBody {
  pub locator: ModuleLocator,
  pub source: String,
}

fn parse_file_pathname(str: &String) -> (&str, &str) {
  lazy_static! {
    static ref RE: Regex = Regex::new(r"^([^/?]+)/(.*)$").unwrap();
  }

  let captures = RE.captures(str.as_ref()).unwrap().unwrap();

  let ns = captures.get(1).unwrap();
  let pathname = captures.get(2).unwrap();

  (ns.as_str(), pathname.as_str())
}
