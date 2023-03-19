use serde::Serialize;
use thiserror::Error;

use crate::types::*;

use super::swc::{ErrorBuffer, extract_diagnostics};

#[derive(Debug, Error, Serialize)]
#[error("Compilation error")]
pub struct CompilationError {
  pub diagnostics: Vec<Diagnostic>,
}

impl CompilationError {
  pub fn from_str(err: &str) -> Self {
    Self {
      diagnostics: [
      Diagnostic {
        message: err.to_string(),
        highlights: vec![].to_vec(),
      },
      ].to_vec(),
    }
  }

  pub fn from_json(err: &serde_json::Error) -> Self {
    Self {
      diagnostics: [
        Diagnostic {
          message: err.to_string(),
          highlights: vec![
            Highlight {
              label: None,
              span: Span {
                start: Position {
                  row: err.line(),
                  col: err.column(),
                },

                end: Position {
                  row: err.line(),
                  col: err.column(),
                },
              }
            },
          ].to_vec(),
        },
      ].to_vec(),
    }
  }

  pub fn from_swc(err: &ErrorBuffer, source_map: &swc_common::SourceMap) -> Self {
    Self {
      diagnostics: extract_diagnostics(err, source_map),
    }
  }

  pub fn from_yaml(err: &serde_yaml::Error) -> Self {
    let highlights = err.location().map(|l| {
      Highlight {
        label: None,
        span: Span {
          start: Position {
            row: l.line(),
            col: l.column(),
          },

          end: Position {
            row: l.line(),
            col: l.column(),
          },
        }
      }
    }).map_or(vec![], |h| {
      vec![h]
    });

    Self {
      diagnostics: [
        Diagnostic {
          message: err.to_string(),
          highlights,
        },
      ].to_vec(),
    }
  }
}
