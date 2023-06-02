use std::sync::Arc;

use swc::config::{SourceMapsConfig, ModuleConfig};
use swc_common::{errors::Handler, GLOBALS, FileName, comments::SingleThreadedComments};
use swc_core::ecma::{ast::EsVersion, parser::{Syntax, TsConfig}, visit::as_folder};

use crate::types::*;
use crate::utils;
use crate::{CompilationError, Project};

mod visitor_1_before;
mod visitor_2_after;

#[derive(Debug, Default, Clone)]
#[napi(object)]
pub struct OnTransformSwcOpts {
  pub use_esfuse_runtime: bool,
  pub promisify_body: bool,
}

pub fn transform_swc(_project: &Project, fetch_data: OnFetchResultData, args: OnTransformArgs) -> OnTransformResult {
  let cm = Arc::<swc_common::SourceMap>::default();
  let c = swc::Compiler::new(cm.clone());

  let mut transform_before = visitor_1_before::TransformVisitor {
  };

  let mut transform_after = visitor_2_after::TransformVisitor {
    opts: &args.opts.swc,
    url: fetch_data.locator.url.clone(),
    imports: vec![],
    try_stack: 0,
  };

  let error_buffer = utils::swc::ErrorBuffer::default();
  let handler = Handler::with_emitter(true, false, Box::new(error_buffer.clone()));

  let file = cm.new_source_file(
    FileName::Custom(fetch_data.locator.url.clone()),
    fetch_data.source.clone(),
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
      "development": true,
      "refresh": true,
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

  let parse_res = c.parse_js(
    file.clone(),
    &handler,
    swc_config.config.jsc.target.unwrap_or(EsVersion::Es2022),
    swc_config.config.jsc.syntax.unwrap(),
    swc::config::IsModule::Bool(true),
    Some(&comments),
  );

  let program = match parse_res {
    Ok(program) => program,

    Err(_) => return OnTransformResult {
      result: Err(CompilationError::from_swc(&error_buffer, fetch_data.locator.url.clone(), &cm)),
      dependencies: vec![
        fetch_data.locator,
      ],
    }
  };

  let transform_res = GLOBALS.set(&Default::default(), || {
    swc_common::errors::HANDLER.set(&handler, || {
      c.run_transform(&handler, true, || {
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
  });

  match transform_res {
    Ok(output) => {
      OnTransformResult {
        result: Ok(OnTransformResultData {
          mime_type: "text/javascript".to_string(),

          code: output.code,
          map: output.map,

          imports: transform_after.imports.into_iter().map(|import_swc| {
            Import {
              kind: import_swc.kind,
              specifier: import_swc.specifier,
              span: Span::from_swc(&import_swc.span, &cm),
              optional: import_swc.optional,
            }
          }).collect(),
        }),
        dependencies: vec![
          fetch_data.locator,
        ],
      }
    },

    Err(_) => {
      OnTransformResult {
        result: Err(CompilationError::from_swc(&error_buffer, fetch_data.locator.url.clone(), &cm)),
        dependencies: vec![
          fetch_data.locator,
        ],
      }
    },
  }
}
