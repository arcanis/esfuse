use fancy_regex::Regex;
use lazy_static::lazy_static;
use serde::Serialize;
use std::path::PathBuf;

use crate::Project;
use crate::utils;

#[derive(Clone, Debug, Default, Serialize)]
#[napi(object)]
pub struct Span {
  pub start: Position,
  pub end: Position,
}

#[derive(Clone, Debug, Default, Serialize)]
#[napi(object)]
pub struct Position {
  pub row: u32,
  pub col: u32,
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
        row: start.line as u32,
        col: (start.col_display + 1) as u32,
      },

      end: Position {
        row: end.line as u32,
        col: end.col_display as u32,
      },
    }
  }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[napi(object)]
pub struct StringKeyValue {
  pub name: String,
  pub value: Option<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModuleLocatorData {
  pub pathname: String,
  pub params: Vec<StringKeyValue>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ModuleLocator {
  Path(ModuleLocatorData),
  DevUrl(ModuleLocatorData),
  ExternalUrl(ModuleLocatorData),
}

impl ModuleLocator {
  pub fn from_url<P: AsRef<str>>(url: P) -> Option<Self> {
    lazy_static! {
      static ref DEV_RE: Regex = Regex::new(r"^/_dev/([^/?]+)/([^?]*)(.*)$").unwrap();
      static ref EXT_RE: Regex = Regex::new(r"^(data:[^?]*)(.*)$").unwrap();
    }

    if let Some(captures) = DEV_RE.captures(url.as_ref()).unwrap() {
      if let (Some(kind), Some(pathname), Some(qs)) = (captures.get(1), captures.get(2), captures.get(3)) {
        let data = ModuleLocatorData {
          pathname: String::from(pathname.as_str()),
          params: utils::parse_query(qs.as_str()),
        };

        return match kind.as_str() {
          "file" => Some(ModuleLocator::Path(data)),
          "internal" => Some(ModuleLocator::DevUrl(data)),
          _ => None
        };
      }
    }

    if let Some(captures) = EXT_RE.captures(url.as_ref()).unwrap() {
      if let (Some(pathname), Some(qs)) = (captures.get(1), captures.get(2)) {
        return Some(ModuleLocator::ExternalUrl(ModuleLocatorData {
          pathname: String::from(pathname.as_str()),
          params: utils::parse_query(qs.as_str()),
        }));
      }
    }

    None
  }

  pub fn without_query(&self) -> Self {
    let mut clone = self.clone();
    clone.params.clear();
    clone
  }

  pub fn url(&self) -> String {
    match &self {
      ModuleLocator::ExternalUrl(data) => {
        format!("{}{}", &data.pathname, utils::stringify_query(&self.params))
      },
      ModuleLocator::Path(data) => {
        format!("/_dev/file/{}{}", &data.pathname, utils::stringify_query(&self.params))
      },
      ModuleLocator::DevUrl(data) => {
        format!("/_dev/internal/{}{}", &data.pathname, utils::stringify_query(&self.params))
      },
    }
  }

  pub fn physical_path(&self, project: &Project) -> Option<PathBuf> {
    match self {
      ModuleLocator::Path(data) => {
        let (ns, pathname) = parse_file_pathname(&data.pathname);
        Some(project.root_ns(ns).join(pathname))
      },

      _ => None,
    }
  }
}

impl std::ops::Deref for ModuleLocator {
  type Target = ModuleLocatorData;
  fn deref(&self) -> &ModuleLocatorData {
    match self {
      ModuleLocator::Path(data) => data,
      ModuleLocator::DevUrl(data) => data,
      ModuleLocator::ExternalUrl(data) => data,
    }
  }
}

impl std::ops::DerefMut for ModuleLocator {
  fn deref_mut(&mut self) -> &mut ModuleLocatorData {
    match self {
      ModuleLocator::Path(data) => data,
      ModuleLocator::DevUrl(data) => data,
      ModuleLocator::ExternalUrl(data) => data,
    }
  }
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
