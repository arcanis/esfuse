pub mod errors;
pub mod swc;

use fancy_regex::Regex;
use lazy_static::lazy_static;
use serde::Serialize;
use serde_json::{Serializer, json};

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

pub type QueryString = Vec<(String, Option<String>)>;

pub fn parse_query(str_opt: Option<String>) -> QueryString {
  let mut params = QueryString::new();

  if let Some(str) = str_opt {
    let slice = match str.starts_with('?') {
      true => &str[1..str.len()],
      false => &str,
    };

    for pair in slice.split('&') {
      if let Some((key, value)) = pair.split_once('=') {
        params.push((key.to_string(), Some(value.to_string())));
      } else {
        params.push((pair.to_string(), None));
      }
    }
  }

  params
}

pub fn stringify_query(params: &QueryString) -> String {
  if params.is_empty() {
    return Default::default();
  }

  let mut str = String::new();
  for (key, val_opt) in params {
    str.push(match str.is_empty() {
      true => '?',
      false => '&',
    });

    str.push_str(&match &val_opt {
      Some(val) => format!("{}={}", urlencoding::encode(key), urlencoding::encode(val)),
      None => urlencoding::encode(key).into_owned(),
    });
  }

  str
}

pub fn serialize_json<T: serde::Serialize>(val: &T) -> Result<String, serde_json::Error> {
  let mut buf = Vec::new();

  let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
  let mut ser = Serializer::with_formatter(&mut buf, formatter);

  json!(&val).serialize(&mut ser).unwrap();

  unsafe {
    // serde_json::to_string promises that the result is always utf8
    Ok(String::from_utf8_unchecked(buf))
  }
}
