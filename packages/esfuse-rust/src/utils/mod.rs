pub use fancy_regex::Regex;
pub mod errors;
pub mod swc;

use std::{pin::Pin, future::Future};

use lazy_static::lazy_static;
use serde::Serialize;
use serde_json::{Serializer, json};

use crate::{types::*, CompilationError};

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
