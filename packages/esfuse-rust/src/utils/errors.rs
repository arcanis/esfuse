use serde::Serialize;
use thiserror::Error;

use crate::types::*;

use super::swc::ErrorBuffer;

#[derive(Clone, Debug, Serialize)]
#[napi(object)]
pub struct Highlight {
  pub source: Option<String>,
  pub subject: Option<String>,
  pub label: Option<String>,
  pub span: Option<Span>,
}

#[derive(Clone, Debug, Serialize)]
#[napi(object)]
pub struct Diagnostic {
  pub message: String,
  pub highlights: Vec<Highlight>,
}

impl Diagnostic {
  pub fn from_str(err: &str) -> Self {
    Self::from_string(err.to_string())
  }

  pub fn from_str_with_highlight(err: &str, highlight: Highlight) -> Self {
    Self::from_string_with_highlight(err.to_string(), highlight)
  }

  pub fn from_string(err: String) -> Self {
    Self {
      message: err,
      highlights: vec![].to_vec(),
    }
  }

  pub fn from_string_with_highlight(err: String, highlight: Highlight) -> Self {
    if highlight.source.is_some() || highlight.subject.is_some() || highlight.label.is_some() || highlight.span.is_some() {
      return Self {
        message: err,
        highlights: vec![
          highlight,
        ],
      }
    }

    Self::from_string(err)
  }

  pub fn from_json(err: &serde_json::Error, source: String) -> Self {
    Self {
      message: err.to_string(),
      highlights: vec![
        Highlight {
          source: Some(source),
          subject: None,
          label: None,
          span: Some(Span {
            start: Position {
              row: err.line() as u32,
              col: err.column() as u32,
            },

            end: Position {
              row: err.line() as u32,
              col: err.column() as u32,
            },
          }),
        },
      ].to_vec(),
    }
  }

  pub fn from_yaml(err: &serde_yaml::Error, source: String) -> Self {
    let highlights = err.location().map(|l| {
      Highlight {
        source: Some(source),
        subject: None,
        label: None,
        span: Some(Span {
          start: Position {
            row: l.line() as u32,
            col: l.column() as u32,
          },

          end: Position {
            row: l.line() as u32,
            col: l.column() as u32,
          },
        }),
      }
    }).map_or(vec![], |h| {
      vec![h]
    });

    Self {
      message: err.to_string(),
      highlights,
    }
  }
}

#[derive(Clone, Debug, Default, Error, Serialize)]
#[error("Compilation error")]
#[napi(object)]
pub struct CompilationError {
  pub diagnostics: Vec<Diagnostic>,
}

impl AsRef<str> for CompilationError {
  fn as_ref(&self) -> &str {
    "Compilation error"
  }
}

impl CompilationError {
  pub fn from_napi(err: napi::Error) -> Self {
    Self::from_str(&err.reason)
  }

  pub fn from_err<T : std::error::Error>(err: T) -> Self {
    Self::from_string(err.to_string())
  }

  pub fn from_str(err: &str) -> Self {
    Self {
      diagnostics: [
        Diagnostic::from_str(err),
      ].to_vec(),
    }
  }

  pub fn from_str_with_highlight(err: &str, highlight: Highlight) -> Self {
    Self {
      diagnostics: [
        Diagnostic::from_str_with_highlight(err, highlight),
      ].to_vec(),
    }
  }

  pub fn from_string(err: String) -> Self {
    Self {
      diagnostics: [
        Diagnostic::from_string(err),
      ].to_vec(),
    }
  }

  pub fn from_string_with_highlight(err: String, highlight: Highlight) -> Self {
    Self {
      diagnostics: [
        Diagnostic::from_string_with_highlight(err, highlight),
      ].to_vec(),
    }
  }

  pub fn from_json(err: &serde_json::Error, subject: String) -> Self {
    Self {
      diagnostics: [
        Diagnostic::from_json(err, subject),
      ].to_vec(),
    }
  }

  pub fn from_swc(err: &ErrorBuffer, source: String, source_map: &swc_common::SourceMap) -> Self {
    let s = err.0.lock().unwrap().clone();

    let diagnostics = s.iter().map(|d| {
      Diagnostic {
        message: d.message(),
        highlights: d.span.span_labels().iter().map(|s| { 
          Highlight {
            source: Some(source.clone()),
            subject: None,
            label: s.label.clone(),
            span: Some(Span::from_swc(&s.span, source_map)),
          }
        }).collect(),
      }
    }).collect();

    Self {
      diagnostics,
    }
  }

  pub fn from_yaml(err: &serde_yaml::Error, subject: String) -> Self {
    Self {
      diagnostics: [
        Diagnostic::from_yaml(err, subject),
      ].to_vec(),
    }
  }
}
