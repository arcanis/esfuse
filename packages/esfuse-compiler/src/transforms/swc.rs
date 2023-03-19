use std::sync::Arc;

use swc::config::{SourceMapsConfig, ModuleConfig};
use swc_common::{errors::Handler, GLOBALS, FileName, comments::SingleThreadedComments};
use swc_core::ecma::{ast::EsVersion, parser::{Syntax, TsConfig}, visit::as_folder};

use crate::types::*;
use crate::{utils};

use super::{TransformError, TransformOutput};

mod visitor_1_before;
mod visitor_2_after;

pub fn transform_swc(module_source: &ModuleBody, _project: &Project) -> Result<TransformOutput, TransformError> {
  let cm = Arc::<swc_common::SourceMap>::default();
  let c = swc::Compiler::new(cm.clone());

  let mut transform_before = visitor_1_before::TransformVisitor {
  };

  let mut transform_after = visitor_2_after::TransformVisitor {
    url: module_source.locator.url(),
    imports: vec![],
  };

  let error_buffer = utils::swc::ErrorBuffer::default();
  let handler = Handler::with_emitter(true, false, Box::new(error_buffer.clone()));

  let output = GLOBALS.set(&Default::default(), || {
    swc_common::errors::HANDLER.set(&handler, || {
      c.run_transform(&handler, true, || {
        let file = cm.new_source_file(
          FileName::Custom(module_source.locator.url()),
          module_source.source.clone(),
        );

        let comments = SingleThreadedComments::default();

        let mut swc_config = swc::config::Options::default();
        swc_config.source_maps = Some(SourceMapsConfig::Bool(true));
        swc_config.config.jsc.target = Some(EsVersion::Es2022);

        swc_config.config.jsc.minify = Some(serde_json::from_str(r#"{
          "compress": false,
          "mangle": false
        }"#).unwrap());

        swc_config.config.module = Some(ModuleConfig::CommonJs(serde_json::from_str(r#"{
          "ignoreDynamic": true
        }"#).unwrap()));

        let syntax = swc_config.config.jsc.syntax.unwrap_or_else(|| {
          Syntax::Typescript(TsConfig {
            tsx: true,
            decorators: true,
            ..Default::default()
          })
        });

        // Need auto detect esm
        let program = c.parse_js(
          file.clone(),
          &handler,
          swc_config.config.jsc.target.unwrap_or(EsVersion::Es2022),
          syntax,
          swc::config::IsModule::Bool(true),
          Some(&comments),
        )?;

        c.process_js_with_custom_pass(
          file,
          Some(program),
          &handler,
          &swc_config,
          comments,
          |_| {
            as_folder(&mut transform_before)
          },
          |_| {
            as_folder(&mut transform_after)
          },
        )
      })
    })
  }).map_err(|_| {
    TransformError::CompilationError(utils::errors::CompilationError::from_swc(&error_buffer, &cm))
  })?;

  Ok(TransformOutput {
    code: output.code,
    map: output.map,
    imports: transform_after.imports,
  })
}
