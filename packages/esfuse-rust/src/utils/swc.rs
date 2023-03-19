use crate::types::*;

#[derive(Debug, Clone, Default)]
pub struct ErrorBuffer(std::sync::Arc<std::sync::Mutex<Vec<swc_common::errors::Diagnostic>>>);

impl swc_common::errors::Emitter for ErrorBuffer {
  fn emit(&mut self, db: &swc_common::errors::DiagnosticBuilder) {
    self.0.lock().unwrap().push((**db).clone());
  }
}

pub fn extract_diagnostics(error_buffer: &ErrorBuffer, source_map: &swc_common::SourceMap) -> Vec<Diagnostic> {
  let s = error_buffer.0.lock().unwrap().clone();

  s.iter().map(|d| {
    Diagnostic {
      message: d.message(),
      highlights: d.span.span_labels().iter().map(|s| { 
        Highlight {
          label: s.label.clone(),
          span: Span::from_swc(&s.span, source_map)
        }
      }).collect(),
    }
  }).collect()
}

