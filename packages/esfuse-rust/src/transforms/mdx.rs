use crate::types::*;
use crate::utils;
use crate::Project;

use super::swc::transform_swc;

pub fn transform_mdx(project: &Project, fetch_data: OnFetchResultData, args: OnTransformArgs) -> OnTransformResult {
  let mdx_res = mdxjs::compile(&fetch_data.source, &mdxjs::Options {
    jsx: true,
    ..Default::default()
  });

  match mdx_res {
    Ok(compiled) => {
      println!("{}", compiled);
      transform_swc(project, OnFetchResultData {
        source: compiled,
        ..fetch_data
      }, args)    
    },

    Err(message) => {
      OnTransformResult {
        result: Err(utils::errors::CompilationError::from_string(message)),
        dependencies: vec![],
      }
    },
  }
}

pub fn transform_mdx_meta(project: &Project, fetch_data: OnFetchResultData, args: OnTransformArgs) -> OnTransformResult {
  let md_res = markdown::to_mdast(&fetch_data.source, &markdown::ParseOptions {
    constructs: markdown::Constructs {
      frontmatter: true,
      ..Default::default()
    },
    ..Default::default()
  });

  match md_res {
    Ok(node) => {
      let meta_node
        = node.children().and_then(|c| c.first());

      let meta = match meta_node {
        Some(markdown::mdast::Node::Yaml(doc)) => Some(doc),
        _ => None,
      };

      let meta_yaml_res = match meta {
        Some(doc) => serde_yaml::from_str::<serde_yaml::Value>(doc.value.as_str()),
        None => Ok(serde_yaml::Value::Mapping(Default::default())),
      };

      match meta_yaml_res {
        Ok(meta_yaml) => {
          let stringified_meta
            = utils::serialize_json(&meta_yaml, &fetch_data.locator.url).unwrap();
          let stringified_url
            = serde_json::to_string(&fetch_data.locator.without_query().url).unwrap();

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
        
          transform_swc(project, OnFetchResultData {
            source: generated,
            ..fetch_data
          }, args)
        },

        Err(_) => {
          OnTransformResult {
            result: Err(utils::errors::CompilationError::from_str("Failed to parse the frontmatter block as valid YAML")),
            dependencies: vec![],
          }
        },
      }
    },

    Err(message) => {
      OnTransformResult {
        result: Err(utils::errors::CompilationError::from_string(message)),
        dependencies: vec![],
      }
    },
  }
}
