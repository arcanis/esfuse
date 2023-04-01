use serde::Serialize;
use thiserror::Error;

use crate::types::*;

use super::swc::ErrorBuffer;

#[derive(Clone, Debug, Serialize)]
#[napi(object)]
pub struct Highlight {
  pub label: Option<String>,
  pub span: Span,
}

#[derive(Clone, Debug, Serialize)]
#[napi(object)]
pub struct Diagnostic {
  pub message: String,
  pub highlights: Vec<Highlight>,
}

impl Diagnostic {
  pub fn from_str(err: &str) -> Self {
    Self {
      message: err.to_string(),
      highlights: vec![].to_vec(),
    }
  }

  pub fn from_string(err: String) -> Self {
    Self {
      message: err,
      highlights: vec![].to_vec(),
    }
  }

  pub fn from_string_with_span(err: String, span: Span) -> Self {
    Self {
      message: err,
      highlights: vec![
        Highlight {
          label: None,
          span
        },
      ].to_vec(),
    }
  }

  pub fn from_json(err: &serde_json::Error) -> Self {
    Self {
      message: err.to_string(),
      highlights: vec![
        Highlight {
          label: None,
          span: Span {
            start: Position {
              row: err.line() as u32,
              col: err.column() as u32,
            },

            end: Position {
              row: err.line() as u32,
              col: err.column() as u32,
            },
          }
        },
      ].to_vec(),
    }
  }

  pub fn from_yaml(err: &serde_yaml::Error) -> Self {
    let highlights = err.location().map(|l| {
      Highlight {
        label: None,
        span: Span {
          start: Position {
            row: l.line() as u32,
            col: l.column() as u32,
          },

          end: Position {
            row: l.line() as u32,
            col: l.column() as u32,
          },
        }
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

#[derive(Debug, Default, Clone, Error)]
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
  pub fn from_str(err: &str) -> Self {
    Self {
      diagnostics: [
        Diagnostic::from_str(err),
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

  pub fn from_string_with_span(err: String, span: Span) -> Self {
    Self {
      diagnostics: [
        Diagnostic::from_string_with_span(err, span),
      ].to_vec(),
    }
  }

  pub fn from_json(err: &serde_json::Error) -> Self {
    Self {
      diagnostics: [
        Diagnostic::from_json(err),
      ].to_vec(),
    }
  }

  pub fn from_swc(err: &ErrorBuffer, source_map: &swc_common::SourceMap) -> Self {
    let s = err.0.lock().unwrap().clone();

    let diagnostics = s.iter().map(|d| {
      Diagnostic {
        message: d.message(),
        highlights: d.span.span_labels().iter().map(|s| { 
          Highlight {
            label: s.label.clone(),
            span: Span::from_swc(&s.span, source_map)
          }
        }).collect(),
      }
    }).collect();

    Self {
      diagnostics,
    }
  }

  pub fn from_yaml(err: &serde_yaml::Error) -> Self {
    Self {
      diagnostics: [
        Diagnostic::from_yaml(err),
      ].to_vec(),
    }
  }
}
