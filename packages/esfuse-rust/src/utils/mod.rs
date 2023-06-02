use arca::Path;
pub use fancy_regex::Regex;
use sha1::{Digest, Sha1};
use std::{pin::Pin, future::Future, collections::HashMap, sync::{Arc, Mutex}};
use lazy_static::lazy_static;
use serde::Serialize;
use serde_json::{Serializer, json};

use crate::{types::*, CompilationError, Project};

pub mod errors;
pub mod swc;

pub type BoxedFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub fn to_slug(str: &str) -> String {
  lazy_static! {
    static ref RE: Regex = Regex::new(r"[^a-z0-9]+").unwrap();
  }

  RE.replace_all(&str.to_lowercase(), "-").into_owned()
}

pub fn get_extension<P: AsRef<str>>(str: P) -> String {
  lazy_static! {
    static ref RE: Regex = Regex::new(r"(?<!^|[\/])(\.[^.]+)$").unwrap();
  }

  let captures = RE.captures(str.as_ref())
    .unwrap_or(None);

  captures
    .and_then(|c| c.get(1))
    .map(|m| m.as_str().to_string())
    .unwrap_or_default()
}

pub struct GetLocatorVirtualPathOpts<'a> {
  pub mime_type: &'a str,
  pub generated_module_folder: &'a Option<Path>,
}

pub fn get_locator_virtual_path(project: &Project, locator: &ModuleLocator, opts: GetLocatorVirtualPathOpts) -> Option<Path> {
  let get_locator_hash_name = || {
    let mut hasher = Sha1::new();
    hasher.update(serde_json::to_string(&locator).unwrap().as_bytes());
    format!("{}{}", hex::encode(hasher.finalize()), get_ext_from_mime(opts.mime_type))
  };

  let module_path = locator.physical_path(&project).map(|p| {
    locator.params.is_empty().then(|| p.clone()).unwrap_or_else(|| {
      p.dirname().join_str(get_locator_hash_name())
    })
  }).unwrap_or_else(|| {
    opts.generated_module_folder.as_ref().unwrap().join_str(get_locator_hash_name())
  });

  Some(module_path)
}

pub fn get_ext_from_mime(ext: &str) -> &str {
  match ext {
    "text/javascript" => {
      ".js"
    }

    "text/markdown" => {
      ".mdx"
    }

    "text/css" => {
      ".css"
    }

    "application/json" => {
      ".json"
    }

    "application/wasm" => {
      ".wasm"
    }

    "image/png" => {
      ".png"
    }

    "image/jpeg" => {
      ".jpg"
    }

    "image/svg+xml" => {
      ".svg"
    }

    _ => {
      ".txt"
    }
  }
}

pub fn get_mime_from_ext(ext: &str) -> &str {
  match ext {
    ".js" | ".cjs" | ".mjs" | ".jsx" | ".ts" | ".cts" | ".mts" | ".tsx" => {
      "text/javascript"
    }

    ".mdx" => {
      "text/markdown"
    }

    ".css" => {
      "text/css"
    }

    ".json" => {
      "application/json"
    }

    ".wasm" => {
      "application/wasm"
    }

    ".png" => {
      "image/png"
    }

    ".jpg" => {
      "image/jpeg"
    }

    ".svg" => {
      "image/svg+xml"
    }

    _ => {
      "text/plain"
    }
  }
}

pub fn is_binary_mime_type<T: AsRef<str>>(mime_type: T) -> bool {
  !mime_type.as_ref().starts_with("text/") && mime_type.as_ref() != "application/json"
}

pub fn split_query(str: &str) -> (&str, Option<&str>) {
  if let Some((subject, query)) = str.split_once('?') {
    (subject, Some(query))
  } else {
    (str, None)
  }
}

pub fn parse_query(str: &str) -> Vec<StringKeyValue> {
  let mut params = Vec::new();

  let slice = match str.starts_with('?') {
    true => &str[1..str.len()],
    false => str,
  };

  if !str.is_empty() {
    for pair in slice.split(&['?', '&']) {
      if let Some((key, value)) = pair.split_once('=') {
        params.push(StringKeyValue {
          name: key.to_string(),
          value: value.to_string(),
        });
      } else {
        params.push(StringKeyValue {
          name: pair.to_string(),
          value: "".to_string(),
        });
      }
    }
  }

  params.sort_by(|a, b| {
    a.name.cmp(&b.name)
  });

  params
}

pub fn stringify_query(params: &Vec<StringKeyValue>) -> String {
  if params.is_empty() {
    return Default::default();
  }

  let mut str = String::new();
  for pair in params {
    str.push(match str.is_empty() {
      true => '?',
      false => '&',
    });

    str.push_str(&match &pair.value.is_empty() {
      false => format!("{}={}", urlencoding::encode(&pair.name), urlencoding::encode(&pair.value)),
      true => urlencoding::encode(&pair.name).into_owned(),
    });
  }

  str
}

pub fn serialize_json<T: serde::Serialize>(val: &T, subject: &String) -> Result<String, CompilationError> {
  let mut buf = Vec::new();

  let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
  let mut ser = Serializer::with_formatter(&mut buf, formatter);

  json!(&val).serialize(&mut ser).unwrap();

  unsafe {
    // serde_json::to_string promises that the result is always utf8
    Ok(String::from_utf8_unchecked(buf))
  }.map_err(|e| CompilationError::from_json(&e, subject.clone()))
}

pub struct FileFinder {
    filename: String,
    cache: Arc<Mutex<HashMap<Path, Option<Path>>>>,
}

impl FileFinder {
    pub fn new<T: AsRef<str>>(filename: T) -> Self {
      FileFinder {
        filename: filename.as_ref().to_string(),
        cache: Arc::new(Mutex::new(HashMap::new())),
      }
    }

    pub fn find_file(&self, starting_directory: &Path) -> Option<Path> {
      let mut cache = self.cache.lock().unwrap();

      // Check if the result is in the cache
      if let Some(cached_path) = cache.get(starting_directory) {
        return cached_path.clone();
      }

      let mut folder_path: Path = starting_directory.clone();
      let filename = Path::from(&self.filename);

      loop {
        let file_path = folder_path.join(&filename);

        if file_path.to_path_buf().is_file() {
          cache.insert(starting_directory.clone(), Some(file_path.clone()));
          break Some(file_path);
        }

        if folder_path.is_root() {
          cache.insert(starting_directory.clone(), None);
          break None;
        }

        folder_path = folder_path.dirname();
      }
    }
}

pub fn interlace_vectors<T>(vec1: Vec<T>, vec2: Vec<T>) -> Vec<T> where T: Clone {
  let mut result: Vec<T> = Vec::new();

  let length1 = vec1.len();
  let length2 = vec2.len();

  let max_length = if length1 > length2 {
    length1
  } else {
    length2
  };

  for i in 0..max_length {
    if i < length1 {
      result.push(vec1[i].clone());
    }
    if i < length2 {
      result.push(vec2[i].clone());
    }
  }

  result
}
