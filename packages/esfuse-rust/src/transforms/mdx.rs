use crate::types::*;
use crate::utils;
use crate::{CompilationError, Project};

use super::swc::transform_swc;

pub fn transform_mdx(project: &Project, module_source: OnFetchResult, args: OnTransformArgs) -> Result<OnTransformResult, CompilationError> {
  let compiled = mdxjs::compile(&module_source.source, &mdxjs::Options {
    ..Default::default()
  }).map_err(|message| {
    utils::errors::CompilationError::from_string(message)
  })?;

  transform_swc(project, OnFetchResult {
    source: compiled,
    ..module_source.clone()
  }, args)
}

pub fn transform_mdx_meta(project: &Project, module_source: OnFetchResult, args: OnTransformArgs) -> Result<OnTransformResult, CompilationError> {
  let compiled = markdown::to_mdast(&module_source.source, &markdown::ParseOptions {
    constructs: markdown::Constructs {
      frontmatter: true,
      ..Default::default()
    },
    ..Default::default()
  }).map_err(|message| {
    utils::errors::CompilationError::from_string(message)
  })?;

  let meta = match compiled.children().and_then(|c| c.first()) {
    Some(markdown::mdast::Node::Yaml(doc)) =>
      serde_yaml::from_str::<serde_yaml::Value>(doc.value.as_str())
        .map_err(|e| utils::errors::CompilationError::from_yaml(&e))?,
    _ =>
    serde_yaml::Value::Mapping(Default::default()),
  };

  let stringified_meta = utils::serialize_json(&meta)
    .map_err(|e| utils::errors::CompilationError::from_json(&e))?;
  let stringified_url = serde_json::to_string(&module_source.locator.without_query().url())
    .map_err(|e| utils::errors::CompilationError::from_json(&e))?;

  let generated = format!(
    concat!(
      "export const meta = {};\n",
      "\n",
      "export async function fetch() {{\n",
      "    const {{default: MDXContent}} = await import({});\n",
      "    return MDXContent;\n",
      "}}\n",
    ),

    stringified_meta,
    stringified_url,
  );

  transform_swc(project, OnFetchResult {
    source: generated,
    ..module_source.clone()
  }, args)
}
