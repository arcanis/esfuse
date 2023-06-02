use std::{sync::Arc, collections::HashMap};

use swc::{config::{SourceMapsConfig, ModuleConfig}, TransformOutput};
use swc_common::{GLOBALS, errors::Handler, FileName, comments::SingleThreadedComments};
use swc_core::ecma::{visit::{as_folder, VisitMut, VisitMutWith}, transforms::base::pass::noop, ast::{EsVersion, self}, utils::quote_str};

use crate::{utils, CompilationError, types::ModuleLocator};

#[derive(Debug, Clone, Default)]
pub struct ErrorBuffer(pub std::sync::Arc<std::sync::Mutex<Vec<swc_common::errors::Diagnostic>>>);

impl swc_common::errors::Emitter for ErrorBuffer {
  fn emit(&mut self, db: &swc_common::errors::DiagnosticBuilder) {
    self.0.lock().unwrap().push((**db).clone());
  }
}

pub fn require_call(e: &mut ast::CallExpr) -> Option<(&mut ast::Expr, String, swc_common::Span)> {
  if let ast::Callee::Expr(callee) = &e.callee {
    if let ast::Expr::Ident(callee_ident) = &**callee {
      if callee_ident.sym.to_string() == "require" {
        if let Some((specifier, span)) = require_param_to_specifier(&e.args[0].expr) {
          return Some((&mut e.args[0].expr, specifier, span));
        }
      }
    }
  }

  None
}

pub fn require_param_to_specifier(e: &ast::Expr) -> Option<(String, swc_common::Span)> {
  match e {
    ast::Expr::Lit(ast::Lit::Str(lit_str)) => {
      Some((lit_str.value.to_string(), lit_str.span))
    }

    ast::Expr::Tpl(tpl) => {
      (tpl.quasis.len() == 1).then(|| {
        let first_quasi = tpl.quasis.first()
          .expect("Should have a quasi");
        
        let quasi_value = first_quasi.cooked.as_ref()
          .expect("Should have a cooked value");
        
        (quasi_value.to_string(), tpl.span)
      })
    }

    _ => {
      None
    }
  }
}

pub struct DependencyUpdater<'a> {
  pub mappings: &'a HashMap<String, String>,
}

impl<'a> DependencyUpdater<'a> {
  pub fn new(mappings: &'a HashMap<String, String>) -> Self {
    Self {mappings}
  }
}

impl<'a> VisitMut for DependencyUpdater<'a> {
  fn visit_mut_call_expr(&mut self, e: &mut ast::CallExpr) {
    e.visit_mut_children_with(self);

    if let Some((expr, specifier, _)) = require_call(e) {
      if let Some(mapping) = self.mappings.get(&specifier) {
        *expr = quote_str!(mapping.as_str()).into();
      }
    }
  }
}

pub fn persist_resolutions(locator: &ModuleLocator, code: &str, map: &str, resolutions: &HashMap<String, String>) -> Result<TransformOutput, CompilationError> {
  let cm = Arc::<swc_common::SourceMap>::default();
  let c = swc::Compiler::new(cm.clone());

  let error_buffer = utils::swc::ErrorBuffer::default();
  let handler = Handler::with_emitter(true, false, Box::new(error_buffer.clone()));

  let comments = SingleThreadedComments::default();

  let file = cm.new_source_file(
    FileName::Anon,
    code.to_string(),
  );

  let mut swc_config = swc::config::Options::default();
  swc_config.config.jsc.target = Some(EsVersion::Es2022);

  swc_config.source_maps = Some(SourceMapsConfig::Bool(true));
  swc_config.config.input_source_map = Some(swc::config::InputSourceMap::Str(map.to_string()));

  swc_config.config.jsc.minify = Some(serde_json::from_str(r#"{
    "compress": false,
    "mangle": false
  }"#).unwrap());

  swc_config.config.module = Some(ModuleConfig::CommonJs(serde_json::from_str(r#"{
    "ignoreDynamic": true
  }"#).unwrap()));

  let mut dependency_updater
    = DependencyUpdater::new(resolutions);

  let transform_res = GLOBALS.set(&Default::default(), || {
    swc_common::errors::HANDLER.set(&handler, || {
      c.run_transform(&handler, true, || {
        c.process_js_with_custom_pass(
          file,
          None,
          &handler,
          &swc_config,
          comments,
          |_| noop(),
          |_| {
            as_folder(&mut dependency_updater)
          },
        )
      })
    })
  });

  transform_res.map_err(|_| {
    CompilationError::from_swc(&error_buffer, locator.url.clone(), &cm)
  })
}
