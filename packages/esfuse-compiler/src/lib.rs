#![deny(clippy::all)]

use std::{sync::Arc};

use swc::{self, config::{self}, try_with_handler, HandlerOpts};
use swc_common::{SourceMap, GLOBALS, FileName, comments::SingleThreadedComments};
use swc_core::ecma::{ast::EsVersion, parser::{TsConfig, Syntax}, visit::as_folder, transforms::base::pass::noop};
use visitor::TransformVisitor;

mod visitor;

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn transform(filename_str: String, code_str: String) -> String {
  let cm = Arc::<SourceMap>::default();
  let c = swc::Compiler::new(cm.clone());

  let output = GLOBALS.set(&Default::default(), || {
    try_with_handler(
      cm.clone(),
      HandlerOpts {
        ..Default::default()
      },
      |handler| {
        c.run_transform(handler, true, || {
          let filename = FileName::Custom(filename_str.clone());

          let file = cm.new_source_file(filename.clone(), code_str);
          let comments = SingleThreadedComments::default();
          
          let mut swc_config = config::Options::default();
          swc_config.filename = filename_str;

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
            handler,
            swc_config.config.jsc.target.unwrap_or(EsVersion::Es2022),
            syntax,
            config::IsModule::Bool(true),
            Some(&comments),
          )?;

          c.process_js_with_custom_pass(
            file,
            Some(program),
            handler,
            &swc_config,
            comments,
            |_| {
              noop()
            },
            |_| {
              as_folder(TransformVisitor {
                filename: filename.clone(),
              })
            },
          )
        })
      }
    )
  }).unwrap();
    
  output.code
}
