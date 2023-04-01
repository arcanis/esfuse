#[derive(Debug, Clone, Default)]
pub struct ErrorBuffer(pub std::sync::Arc<std::sync::Mutex<Vec<swc_common::errors::Diagnostic>>>);

impl swc_common::errors::Emitter for ErrorBuffer {
  fn emit(&mut self, db: &swc_common::errors::DiagnosticBuilder) {
    self.0.lock().unwrap().push((**db).clone());
  }
}
