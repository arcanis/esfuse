use lightningcss::stylesheet::MinifyOptions;
use lightningcss::stylesheet::ParserOptions;

use crate::CompilationError;
use crate::types::*;
use crate::utils;
use crate::Project;

use super::OnTransformResult;
use super::swc::transform_swc;

pub fn transform_css(_project: &Project, fetch_data: OnFetchResultData, _args: OnTransformArgs) -> OnTransformResult {
  let css_modules_config = lightningcss::css_modules::Config {
    pattern: Default::default(),
    dashed_idents: false,
  };

  let parse_res = lightningcss::stylesheet::StyleSheet::parse(&fetch_data.source, ParserOptions {
    css_modules: Some(css_modules_config),
    ..Default::default()
  }).map_err(CompilationError::from_err);

  let minify_res = parse_res.and_then(|mut style_sheet| {
    style_sheet.minify(MinifyOptions::default())
      .map(|_| style_sheet)
      .map_err(CompilationError::from_err)
  });

  let codegen_res = minify_res.and_then(|style_sheet| {
    style_sheet.to_css(Default::default()).map_err(|err| {
      CompilationError::from_string(err.to_string())
    })
  });

  return match codegen_res {
    Ok(codegen) => {
      OnTransformResult {
        result: Ok(OnTransformResultData {
          mime_type: "text/css".to_string(),

          code: codegen.code,
          map: None,

          imports: vec![],
        }),
        dependencies: vec![],
      }
    },

    Err(err) => {
      OnTransformResult {
        result: Err(CompilationError::from_string(err.to_string())),
        dependencies: vec![],
      }
    },
  }
}

pub fn transform_css_js(project: &Project, fetch_data: OnFetchResultData, args: OnTransformArgs) -> OnTransformResult {
  let css_modules_config = lightningcss::css_modules::Config {
    pattern: Default::default(),
    dashed_idents: false,
  };

  let parse_res = lightningcss::stylesheet::StyleSheet::parse(&fetch_data.source, ParserOptions {
    css_modules: fetch_data.locator.specifier.ends_with(".module.css").then_some(css_modules_config),
    ..Default::default()
  }).map_err(CompilationError::from_err);

  let minify_res = parse_res.and_then(|mut style_sheet| {
    style_sheet.minify(MinifyOptions::default())
      .map(|_| style_sheet)
      .map_err(CompilationError::from_err)
  });

  let codegen_res = minify_res.and_then(|style_sheet| {
    style_sheet.to_css(Default::default()).map_err(|err| {
      CompilationError::from_string(err.to_string())
    })
  });

  return match codegen_res {
    Ok(codegen) => {
      let stringified_code
        = serde_json::to_string(&codegen.code)
          .map_err(|e| utils::errors::CompilationError::from_json(&e, fetch_data.locator.url.clone()))
          .unwrap();

      let mut generated = format!(
        concat!(
          "if (typeof document !== 'undefined') {{\n",
          "  let node = document.head.querySelector(`.esfuse-css-module[data-name=\"${{module.id}}\"]`);\n",
          "\n",
          "  if (!node) {{\n",
          "    node = document.createElement('style');\n",
          "    node.type = 'text/css';\n",
          "    node.classList.add('esfuse-css-module');\n",
          "    node.setAttribute('data-name', module.id);\n",
          "    document.head.appendChild(node);\n",
          "  }}\n",
          "\n",
          "  node.textContent = {};\n",
          "}}\n",
          "\n",
        ),
    
        stringified_code,
      );

      if let Some(exports) = codegen.exports {
        generated.push_str("\nexport const styles = {\n");
        for (name, export) in exports.iter() {
          let stringified_js_name
            = serde_json::to_string(&name)
              .map_err(|e| utils::errors::CompilationError::from_json(&e, fetch_data.locator.url.clone()))
              .unwrap();

          let stringified_css_name
            = serde_json::to_string(&export.name)
              .map_err(|e| utils::errors::CompilationError::from_json(&e, fetch_data.locator.url.clone()))
              .unwrap();
    
          generated.push_str(format!("\n  [{}]: {},\n", stringified_js_name, stringified_css_name).as_str());
        }
        generated.push_str("\n};\n");
      }
    
      transform_swc(project, OnFetchResultData {
        source: generated,
        ..fetch_data
      }, args)
    },

    Err(err) => {
      OnTransformResult {
        result: Err(CompilationError::from_string(err.to_string())),
        dependencies: vec![],
      }
    },
  }
}
