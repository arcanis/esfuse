use std::mem::swap;

use html5ever::tendril::TendrilSink;
use lol_html::errors::RewritingError;
use lol_html::html_content::ContentType;
use lol_html::{element, HtmlRewriter, Settings};

use html5ever::{parse_document, serialize, QualName};


use crate::actions::fetch::OnFetchResult;
use crate::actions::resolve::{OnResolveArgs, Resolution, resolve};
use crate::types::*;
use crate::{CompilationError, Project};

use super::{OnTransformArgs, OnTransformResult};

#[derive(Default)]
#[napi(object)]
pub struct HtmlAttribute {
  pub name: String,
  pub value: String,
}

#[derive(Default)]
#[napi(object)]
pub struct HtmlReference {
  pub src: String,
  pub attributes: Vec<HtmlAttribute>,
}

#[derive(Default)]
#[napi(object)]
pub struct OnTransformHtmlArgs {
  pub stylesheets: Vec<HtmlReference>,
  pub scripts: Vec<HtmlReference>,
}

pub async fn transform_html(module_source: &OnFetchResult, project: &Project, opts: &OnTransformArgs) -> Result<OnTransformResult, CompilationError> {
  let module_path = module_source.locator.physical_path(project)
    .unwrap_or(project.root.join("{file.ext}"));

  let resolve_assets_opts = OnResolveArgs {
    force_params: vec![].into(),
  };

  let resolve_scripts_opts = OnResolveArgs {
    force_params: vec![StringKeyValue {
      name: String::from("transform"),
      value: Some(String::from("js")),
    }].into(),
  };

  html5ever::parse_document(html5ever::rcdom::RcDom {}, opts);

  let document = kuchiki::parse_html()
    .one(module_source.source.clone());

  let head = document.select_first("head")
    .map_err(|_| CompilationError::from_str("Missing head tag"))?
    .as_node();

  for stylesheet in &opts.html.stylesheets {
    let qual_name = html5ever::QualName::new(None, html5ever::ns!(html), html5ever::LocalName::from("foo"));
    let stylesheet = kuchiki::NodeRef::new_element(qual_name, vec![]);
    head.append(new_child);
  }

  head.attributes.borrow_mut().insert("foo", String::from("bar"));

  let code = document.to_string();

/*
  let mut output = vec![];

  let mut rewriter = HtmlRewriter::new(
    Settings {
      element_content_handlers: vec![
        element!("head", |el| {
          let mut html = String::new();

          for stylesheet in &opts.html.stylesheets {
            html.push_str("<link rel=\"stylesheet\" href=\"");

            match resolve(project, &stylesheet.src, &module_path, Default::default(), &resolve_assets_opts).await.result {
              Ok(Resolution::Module(locator)) => {
                html_escape::encode_double_quoted_attribute_to_string(locator.url(), &mut html);
              },
              Err(err) => {
                return Err(Box::from(err))
              },
            };

            for attr in &stylesheet.attributes {
              html.push_str("\" ");
              html.push_str(&attr.name);
              html.push_str("=\"");
              html_escape::encode_double_quoted_attribute_to_string(&attr.value, &mut html);
            }

            html.push_str("\"/>\n");
          }

          for script in &opts.html.scripts {
            html.push_str("<script defer src=\"");

            match resolve(project, &script.src, &module_path, Default::default(), &resolve_scripts_opts).result {
              Ok(Resolution::Module(locator)) => {
                html_escape::encode_double_quoted_attribute_to_string(locator.url(), &mut html);
              },
              Err(err) => {
                return Err(Box::from(err))
              },
            };

            for attr in &script.attributes {
              html.push_str("\" ");
              html.push_str(&attr.name);
              html.push_str("=\"");
              html_escape::encode_double_quoted_attribute_to_string(&attr.value, &mut html);
            }

            html.push_str("\"></script>\n");
          }

          el.prepend(&html, ContentType::Html);

          Ok(())
        }),

        element!("link[href]", |el| {
          let href = el.get_attribute("href").unwrap();
          let resolution = resolve(project, &href, &module_path, Default::default(), &resolve_assets_opts);

          match resolution.result {
            Ok(Resolution::Module(locator)) => {
              el.set_attribute("href", &locator.url())?;
              return Ok(())
            }

            Err(err) => {
              return Err(Box::from(err))
            }
          }
        }),

        element!("script[src]", |el| {
          let href = el.get_attribute("src").unwrap();
          let resolution = resolve(project, &href, &module_path, Default::default(), &resolve_scripts_opts);

          match resolution.result {
            Ok(Resolution::Module(locator)) => {
              el.set_attribute("src", &locator.url())?;
              return Ok(())
            }

            Err(err) => {
              return Err(Box::from(err))
            }
          }
        }),
      ],
      ..Settings::default()
    },
    |c: &[u8]| output.extend_from_slice(c)
  );

  if let Err(mut err_html) = rewriter.write(module_source.source.as_bytes()) {
    return Err(extract_compilation_error(&mut err_html));
  }

  if let Err(mut err_html) = rewriter.end() {
    return Err(extract_compilation_error(&mut err_html));
  }

  let code = String::from_utf8(output)
    .unwrap();
*/

  Ok(OnTransformResult {
    mime_type: "text/html".to_string(),

    code,
    map: None,

    imports: vec![],
  })
}
