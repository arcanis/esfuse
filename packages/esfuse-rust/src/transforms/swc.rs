use std::sync::Arc;

use swc::config::{SourceMapsConfig, ModuleConfig};
use swc_common::{errors::Handler, GLOBALS, FileName, comments::SingleThreadedComments};
use swc_core::ecma::{ast::EsVersion, parser::{Syntax, TsConfig}, visit::as_folder};

use crate::actions::fetch::OnFetchResult;
use crate::types::*;
use crate::utils;
use crate::{CompilationError, Project};

use super::{OnTransformArgs, OnTransformResult};

mod visitor_1_before;
mod visitor_2_after;

#[derive(Default)]
#[napi(object)]
pub struct OnTransformSwcArgs {
  pub use_esfuse_runtime: bool,
}

pub fn transform_swc(module_source: &OnFetchResult, _project: &Project, opts: &OnTransformArgs) -> Result<OnTransformResult, CompilationError> {
  let cm = Arc::<swc_common::SourceMap>::default();
  let c = swc::Compiler::new(cm.clone());

  let mut transform_before = visitor_1_before::TransformVisitor {
  };

  let mut transform_after = visitor_2_after::TransformVisitor {
    opts: &opts.swc,
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

        swc_config.config.jsc.transform = Some(serde_json::from_str(r#"{
          "optimizer": {
            "globals": {
              "envs": {
                "NODE_ENV": "\"development\""
              }
            }
          },
          "react": {
            "runtime": "automatic"
          }
        }"#).unwrap()).into();

        swc_config.config.jsc.minify = Some(serde_json::from_str(r#"{
          "compress": false,
          "mangle": false
        }"#).unwrap());

        swc_config.config.module = Some(ModuleConfig::CommonJs(serde_json::from_str(r#"{
          "ignoreDynamic": true
        }"#).unwrap()));

        swc_config.config.jsc.syntax = Some(Syntax::Typescript(TsConfig {
          tsx: true,
          decorators: true,
          ..Default::default()
        }));

        let program = c.parse_js(
          file.clone(),
          &handler,
          swc_config.config.jsc.target.unwrap_or(EsVersion::Es2022),
          swc_config.config.jsc.syntax.unwrap(),
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
    CompilationError::from_swc(&error_buffer, &cm)
  })?;

  Ok(OnTransformResult {
    mime_type: "text/javascript".to_string(),

    code: output.code,
    map: output.map,

    imports: transform_after.imports.into_iter().map(|(import, swc_span)| {
      super::ExtractedImport {
        specifier: import,
        span: Span::from_swc(&swc_span, &cm),
      }
    }).collect(),
  })
}
