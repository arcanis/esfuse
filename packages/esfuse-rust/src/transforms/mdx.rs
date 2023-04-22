use serde::Serialize;
use serde_json::json;

use crate::CompilationError;
use crate::types::*;
use crate::utils;
use crate::Project;

use super::swc::transform_swc;

#[derive(Serialize)]
struct TableOfContent {
  id: String,
  name: String,
  children: Vec<TableOfContent>,
}

fn transform_result_err_from_string(message: String) -> OnTransformResult {
  OnTransformResult {
    result: Err(utils::errors::CompilationError::from_string(message)),
    dependencies: vec![],
  }
}

fn get_element_heading_level(element: &mdxjs::hast::Element) -> Option<u32> {
  match element.tag_name.as_str() {
    "h1" => Some(1),
    "h2" => Some(2),
    "h3" => Some(3),
    "h4" => Some(4),
    "h5" => Some(5),
    _ => None,
  }
}

fn get_node_text(element: &mdxjs::hast::Node) -> String {
  match element {
    mdxjs::hast::Node::Element(element) => get_element_text(element),
    mdxjs::hast::Node::Text(text) => text.value.clone(),
    _ => String::new(),
  }
}

fn get_element_text(element: &mdxjs::hast::Element) -> String {
  element.children.iter()
    .map(|n| get_node_text(n))
    .collect::<Vec<String>>()
    .join("")
}

fn flat_to_tree(input: Vec<(u32, TableOfContent)>) -> Option<TableOfContent> {
  fn build_tree(input: &mut std::iter::Peekable<std::slice::Iter<(u32, TableOfContent)>>, depth: u32) -> Option<TableOfContent> {
    let (current_depth, value) = input.peek()?.clone();
    if *current_depth < depth {
      return None;
    }

    input.next();

    let mut children = vec![];
    while let Some(child) = build_tree(input, current_depth + 1) {
      children.push(child);
    }

    Some(TableOfContent {
      id: value.id.clone(),
      name: value.name.clone(),
      children,
    })
  }

  let mut iter = input.iter().peekable();
  let depth = iter.peek()?.0;
  build_tree(&mut iter, depth)
}

fn get_table_of_content(node: &mut mdxjs::hast::Node) -> Vec<TableOfContent> {
  let mut headings = vec![];

  headings.push((0, TableOfContent {
    id: String::new(),
    name: String::new(),
    children: vec![],
  }));

  if let Some(children) = node.children_mut() {
    for child in children {
      if let mdxjs::hast::Node::Element(element) = child {
        if let Some(level) = get_element_heading_level(element) {
          let element_text = get_element_text(element);
          let element_id = utils::to_slug(&element_text);

          let toc_entry = TableOfContent {
            id: element_id.clone(),
            name: element_text,
            children: vec![],
          };

          headings.push((level, toc_entry));

          element.properties.push((
            String::from("id"),
            mdxjs::hast::PropertyValue::String(element_id),
          ));
        }
      }
    }
  }

  flat_to_tree(headings).unwrap().children
}

fn get_meta(node: &markdown::mdast::Node) -> Result<serde_json::Value, CompilationError> {
  let meta_node
    = node.children().and_then(|c| c.first());

  let meta_yaml = match meta_node {
    Some(markdown::mdast::Node::Yaml(doc)) => Some(doc),
    _ => None,
  };

  let meta_res = match meta_yaml {
    Some(doc) => serde_yaml::from_str::<serde_json::Value>(doc.value.as_str()),
    None => Ok(serde_json::Value::Object(Default::default())),
  };

  meta_res
    .map_err(|_| CompilationError::from_str("Failed to parse the frontmatter block as valid YAML"))
}

fn mdx_to_hast(fetch_data: &OnFetchResultData) -> Result<(mdxjs::hast::Node, serde_json::Value, mdxjs::Options), CompilationError> {
  let options = mdxjs::Options {
    parse: mdxjs::MdxParseOptions {
      constructs: mdxjs::MdxConstructs {
        frontmatter: true,
        ..Default::default()
      },
      ..Default::default()
    },
    jsx: true,
    ..Default::default()
  };

  let mdast = match mdxjs::parse_to_mdast(&fetch_data.source, &options) {
    Ok(node) => node,
    Err(err) => return Err(CompilationError::from_string(err)),
  };

  let mut hast = mdxjs::mdast_to_hast(&mdast);
  let toc = get_table_of_content(&mut hast);

  let mut meta = get_meta(&mdast)?;
  if let serde_json::Value::Object(meta_obj) = &mut meta {
    meta_obj.insert(String::from("toc"), json!(toc));
  }

  Ok((hast, meta, options))
}

pub fn transform_mdx(project: &Project, fetch_data: OnFetchResultData, args: OnTransformArgs) -> OnTransformResult {
  let (hast, meta, options) = match mdx_to_hast(&fetch_data) {
    Err(err) => return OnTransformResult {result: Err(err), dependencies: vec![]},
    Ok(res) => res,
  };

  let mut program = match mdxjs::hast_to_program(&hast, &fetch_data.source, &options) {
    Err(err) => return transform_result_err_from_string(err),
    Ok(node) => node,
  };

  let compiled: String = mdxjs::program_to_string(&mut program);

  transform_swc(project, OnFetchResultData {
    source: format!("{}\nexport const meta = {};\n", compiled, meta),
    ..fetch_data
  }, args)
}

pub fn transform_mdx_meta(project: &Project, fetch_data: OnFetchResultData, args: OnTransformArgs) -> OnTransformResult {
  let (_hast, meta, _options) = match mdx_to_hast(&fetch_data) {
    Err(err) => return OnTransformResult {result: Err(err), dependencies: vec![]},
    Ok(res) => res,
  };

  let url
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

    meta,
    url,
  );

  transform_swc(project, OnFetchResultData {
    source: generated,
    ..fetch_data
  }, args)
}
