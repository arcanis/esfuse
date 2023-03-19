use crate::types::*;
use crate::utils;

use super::{TransformError, TransformOutput};
use super::swc::transform_swc;

pub fn transform_mdx(module_source: &ModuleBody, project: &Project) -> Result<TransformOutput, TransformError> {
  let compiled = mdxjs::compile(&module_source.source, &mdxjs::Options {
    ..Default::default()
  }).map_err(|message| {
    TransformError::CompilationError(utils::errors::CompilationError::from_str(&message))
  })?;

  transform_swc(&ModuleBody {
    source: compiled,
    ..module_source.clone()
  }, project)
}

pub fn transform_mdx_meta(module_source: &ModuleBody, project: &Project) -> Result<TransformOutput, TransformError> {
  let compiled = markdown::to_mdast(&module_source.source, &markdown::ParseOptions {
    constructs: markdown::Constructs {
      frontmatter: true,
      ..Default::default()
    },
    ..Default::default()
  }).map_err(|message| {
    TransformError::CompilationError(utils::errors::CompilationError::from_str(&message))
  })?;

  let meta = match compiled.children().and_then(|c| c.first()) {
    Some(markdown::mdast::Node::Yaml(doc)) =>
    serde_yaml::from_str::<serde_yaml::Value>(doc.value.as_str())
    .map_err(|e| TransformError::CompilationError(utils::errors::CompilationError::from_yaml(&e)))?,
    _ =>
    serde_yaml::Value::Mapping(Default::default()),
  };

  let stringified_meta = utils::serialize_json(&meta)
  .map_err(|e| TransformError::CompilationError(utils::errors::CompilationError::from_json(&e)))?;
  let stringified_url = serde_json::to_string(&module_source.locator.without_query().url())
  .map_err(|e| TransformError::CompilationError(utils::errors::CompilationError::from_json(&e)))?;

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

  transform_swc(&ModuleBody {
    source: generated,
    ..module_source.clone()
  }, project)
}
