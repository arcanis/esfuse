use fancy_regex::Regex;
use lazy_static::lazy_static;
use napi::bindgen_prelude::FromNapiValue;
use napi::bindgen_prelude::ToNapiValue;
use parcel_sourcemap::SourceMap;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::CompilationError;
use crate::Project;
use crate::transforms::OnTransformSwcOpts;
use crate::utils;

#[derive(Debug)]
#[napi]
pub enum ResolutionKind {
  ImportDeclaration,
  DynamicImport,
}

#[derive(Debug, Default, Clone)]
#[napi(object)]
pub struct OnResolveOpts {
  pub force_params: Vec<StringKeyValue>,
  pub user_data: Arc<serde_json::Value>,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct OnResolveArgs {
  pub kind: ResolutionKind,
  pub request: String,
  pub issuer: Option<ModuleLocator>,
  pub span: Option<Span>,
  pub opts: OnResolveOpts,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct OnResolveResultData {
  pub locator: ModuleLocator,
}

#[derive(Debug, Clone)]
pub struct OnResolveResult {
  pub result: Result<OnResolveResultData, CompilationError>,
  pub dependencies: Vec<ModuleLocator>,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct OnFetchArgs {
  pub locator: ModuleLocator,
  pub opts: OnFetchOpts,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct OnFetchOpts {
  pub user_data: Arc<serde_json::Value>,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct OnFetchResultData {
  pub locator: ModuleLocator,
  pub mime_type: String,
  pub source: String,
}

#[derive(Debug, Clone)]
pub struct OnFetchResult {
  pub result: Result<OnFetchResultData, CompilationError>,
  pub dependencies: Vec<ModuleLocator>,
}

#[derive(Debug, Default, Clone)]
#[napi(object)]
pub struct OnTransformOpts {
  pub swc: OnTransformSwcOpts,
  pub static_resolutions: HashMap<String, String>,
  pub user_data: Arc<serde_json::Value>,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct OnTransformArgs {
  pub locator: ModuleLocator,
  pub opts: OnTransformOpts,
}

#[derive(Debug, Clone)]
pub struct ImportSwc {
  pub kind: ResolutionKind,
  pub specifier: String,
  pub span: swc_common::Span,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct Import {
  pub kind: ResolutionKind,
  pub specifier: String,
  pub span: Span,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct OnTransformResultData {
  pub mime_type: String,

  pub code: String,
  pub map: Option<String>,

  pub imports: Vec<Import>,
}

#[derive(Debug, Clone)]
pub struct OnTransformResult {
  pub result: Result<OnTransformResultData, CompilationError>,
  pub dependencies: Vec<ModuleLocator>,
}

#[derive(Debug, Default, Clone)]
#[napi(object)]
pub struct OnBatchOpts {
  pub promisify_entry_point: bool,
  pub use_esfuse_runtime: bool,
  pub user_data: Arc<serde_json::Value>,
  pub traverse_dependencies: bool,
  pub traverse_packages: bool,
  pub traverse_vendors: bool,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct OnBatchArgs {
  pub locators: Vec<ModuleLocator>,
  pub opts: OnBatchOpts,
}

#[derive(Debug, Clone)]
pub struct OnBatchModule {
  pub locator: ModuleLocator,

  pub mime_type: String,
  pub code: String,

  pub map: Option<SourceMap>,

  pub newlines: usize,
  pub resolutions: HashMap<String, Option<String>>,
}

impl OnBatchModule {
  pub fn new(locator: ModuleLocator, transform: OnTransformResultData, resolutions: HashMap<String, Option<String>>) -> Self {
    let map = transform.map.map(|str| {
      parcel_sourcemap::SourceMap::from_json("/", &str)
        .expect("Assertion failed: Expected the SWC-generated sourcemap to be readable")
    });

    let newlines = count_newlines(transform.code.as_str());

    Self {
      locator,
      mime_type: transform.mime_type,
      code: transform.code,
      map,
      newlines,
      resolutions,
    }
  }
}

#[derive(Debug)]
pub struct OnBatchModuleResult {
  pub result: Result<OnBatchModule, CompilationError>,
  pub dependencies: Vec<ModuleLocator>,
}

pub struct OnBatchResult {
  pub results: HashMap<String, OnBatchModuleResult>,
}

#[derive(Debug, Default, Clone)]
#[napi(object)]
pub struct OnBundleOpts {
  pub batch: OnBatchOpts,
  pub require_on_load: bool,
  pub runtime: Option<ModuleLocator>,
}

#[napi(object)]
pub struct OnBundleArgs {
  pub locator: ModuleLocator,
  pub opts: OnBundleOpts,
}

#[derive(Clone, Serialize)]
#[napi(object)]
pub struct OnBundleModuleMeta {
  pub error: Option<CompilationError>,
  pub path: Option<String>,
  pub resolutions: HashMap<String, Option<String>>,
}

#[derive(Clone)]
#[napi(object)]
pub struct OnBundleResultData {
  pub entry: String,
  pub mime_type: String,

  pub code: String,
  pub map: String,
}

#[derive(Clone)]
pub struct OnBundleResult {
  pub result: Result<OnBundleResultData, CompilationError>,
  pub dependencies: Vec<ModuleLocator>,
}

pub type PluginData = Box<dyn std::any::Any + Send + Sync>;

pub struct PluginHook<TArgs, TRes> {
  pub regexp: Regex,
  pub params: Vec<OptionStringKeyValue>,
  pub cb: fn (registration_data: Arc<PluginData>, args: TArgs) -> utils::BoxedFuture<Option<TRes>>,
  pub data: Arc<PluginData>,
}

pub type OnResolveHook = fn (data: Arc<PluginData>, args: OnResolveArgs)
  -> utils::BoxedFuture<Option<OnResolveResult>>;
pub type OnFetchHook = fn (data: Arc<PluginData>, args: OnFetchArgs)
  -> utils::BoxedFuture<Option<OnFetchResult>>;

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

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[napi(object)]
pub struct StringKeyValue {
  pub name: String,
  pub value: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[napi(object)]
pub struct OptionStringKeyValue {
  pub name: String,
  pub value: Option<String>,
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
#[napi]
pub enum ModuleLocatorKind {
  File,
  Internal,
  External,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[napi(object)]
pub struct ModuleLocator {
  pub url: String,
  pub kind: ModuleLocatorKind,
  pub specifier: String,
  pub params: Vec<StringKeyValue>,
}

impl ModuleLocator {
  pub fn new(kind: ModuleLocatorKind, specifier: String, params: Vec<StringKeyValue>) -> Self {
    let url = match kind {
      ModuleLocatorKind::File => {
        format!("/_dev/file/{}{}", &specifier, utils::stringify_query(&params))
      },
      ModuleLocatorKind::Internal => {
        format!("/_dev/internal/{}{}", &specifier, utils::stringify_query(&params))
      },
      ModuleLocatorKind::External => {
        format!("{}{}", &specifier, utils::stringify_query(&params))
      },
    };

    Self { url, kind, specifier, params }
  }

  pub fn from_url<P: AsRef<str>>(url: P) -> Option<Self> {
    lazy_static! {
      static ref DEV_RE: Regex = Regex::new(r"^/_dev/([^/?]+)/([^?]*)(.*)$").unwrap();
    }

    if let Some(captures) = DEV_RE.captures(url.as_ref()).unwrap() {
      if let (Some(kind_segment), Some(specifier_segment), Some(qs_segment)) = (captures.get(1), captures.get(2), captures.get(3)) {
        let kind = match kind_segment.as_str() {
          "file" => ModuleLocatorKind::File,
          "internal" => ModuleLocatorKind::Internal,
          _ => return None,
        };

        let specifier = String::from(specifier_segment.as_str());
        let params = utils::parse_query(qs_segment.as_str());

        return Some(ModuleLocator::new(kind, specifier, params));
      }
    }

    None
  }

  pub fn without_query(&self) -> Self {
    Self::new(self.kind, self.specifier.clone(), vec![])
  }

  pub fn physical_path(&self, project: &Project) -> Option<PathBuf> {
    match self.kind {
      ModuleLocatorKind::File => {
        Some(project.path_from_ns_qualified(&self.specifier))
      },

      _ => None,
    }
  }
}

fn count_newlines(s: &str) -> usize {
  s.as_bytes().iter().filter(|&&c| c == b'\n').count()
}
