use crate::types::*;
use crate::Project;

mod css;
mod mdx;
mod swc;

pub use self::swc::OnTransformSwcOpts;

pub fn transform(project: &Project, fetch_data: OnFetchResultData, args: OnTransformArgs) -> OnTransformResult {
  match fetch_data.mime_type.as_str() {
    "text/css" => {
      if fetch_data.locator.params.iter().any(|pair| pair.name == "transform" && pair.value == "js") {
        self::css::transform_css_js(project, fetch_data, args)
      } else {
        self::css::transform_css(project, fetch_data, args)
      }
    }

    "text/javascript" => {
      self::swc::transform_swc(project, fetch_data, args)
    }

    "text/markdown" => {
      if fetch_data.locator.params.iter().any(|pair| pair.name == "meta") {
        self::mdx::transform_mdx_meta(project, fetch_data, args)
      } else {
        self::mdx::transform_mdx(project, fetch_data, args)
      }
    }

    _ => {
      OnTransformResult {
        result: Ok(OnTransformResultData {
          mime_type: fetch_data.mime_type,
          code: fetch_data.source,
          map: None,
          imports: vec![],
        }),
        dependencies: vec![],
      }
    }
  }
}


#[derive(Debug)]
pub struct Tree<T> {
  value: T,
  children: Vec<Tree<T>>,
}

impl<T> Tree<T> {
  pub fn new(value: T) -> Self {
      Tree {
          value,
          children: vec![],
      }
  }
}

#[derive(Clone, Copy, Debug)]
pub struct Value {
  pub value: i32,
}

fn flat_to_tree(input: Vec<(u32, Value)>) -> Option<Tree<Value>> {
  fn build_tree<T>(input: &mut std::iter::Peekable<std::slice::Iter<(u32, T)>>, depth: u32) -> Option<Tree<T>>
  where
      T: Clone,
  {
      let (current_depth, value) = input.peek()?.clone();
      if *current_depth < depth {
          return None;
      }
      input.next();

      let mut children = vec![];
      while let Some(child) = build_tree(input, current_depth + 1) {
          children.push(child);
      }

      Some(Tree {
          value: value.clone(),
          children,
      })
  }

  let mut iter = input.iter().peekable();
  let depth = iter.peek()?.0;
  build_tree(&mut iter, depth)
}

fn main() {
  let flat = vec![
      (0, Value { value: 1 }),
      (1, Value { value: 2 }),
      (1, Value { value: 3 }),
      (2, Value { value: 4 }),
      (2, Value { value: 5 }),
      (1, Value { value: 6 }),
  ];

  let tree = flat_to_tree(flat);
  println!("{:#?}", tree);
}
