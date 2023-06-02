extern crate queues;

use itertools::Itertools;
use std::collections::HashMap;
use std::sync::Arc;

use crate::utils;
use crate::Project;
use crate::types::*;

use super::batch::batch;

pub async fn bundle(project_base: Arc<Project>, args: OnBundleArgs) -> OnBundleResult {
  let project = project_base.as_ref();

  let mut final_nl_count = 0_usize;
  let mut final_source = String::default();
  let mut final_source_map = parcel_sourcemap::SourceMap::new("");

  if let Some(runtime_locator) = &args.opts.runtime {
    let runtime_res = super::transform::transform(project, OnTransformArgs {
      locator: runtime_locator.clone(),
      opts: Default::default(),
    }).await;

    match runtime_res.result {
      Ok(runtime) => {
        final_nl_count += count_newlines(&runtime.code);
        final_source.push_str(&runtime.code);
        final_source.push(' ');
      },

      Err(err) => {
        return OnBundleResult {
          result: Err(err),
          dependencies: vec![],
        };
      },
    }
  };

  let mut build_results = batch(project_base.clone(), OnBatchArgs {
    locators: vec![args.locator.clone()],
    opts: args.opts.batch,
  }).await;

  let mut sorted_results: Vec<(String, OnBatchModuleResult)>
    = build_results.results.drain().collect();

  sorted_results.sort_by(|a, b| {
    b.0.cmp(&a.0)
  });

  let mut meta = HashMap::new();

  for (url, result) in sorted_results {
    if final_nl_count > 0 {
      final_nl_count += 1;
      final_source += "\n";
    }
  
    match result.result {
      Ok(module) => {
        if let Some(mut map) = module.map {
          final_source_map.add_sourcemap(&mut map, final_nl_count as i64)
            .expect("Assertion failed: Expected the SWC-generated sourcemap to be well-structured");  
        }

        final_nl_count += module.newlines;
        final_source += module.code.as_str();

        meta.insert(url, OnBundleModuleMeta {
          error: None,
          path: result.locator.physical_path(project),
          resolutions: HashMap::from_iter(module.resolutions.into_iter().map(|(k, v)| {
            (k, v.map(|l| l.url))
          }).collect_vec()),
        });
      },

      Err(err) => {
        meta.insert(url, OnBundleModuleMeta {
          error: Some(err),
          path: None,
          resolutions: Default::default(),
        });
      },
    };
  }

  if final_nl_count > 0 {
    final_source += "\n";
  }
  
  final_source += format!("$esfuse$.meta({});\n", utils::serialize_json(&meta, &args.locator.url).unwrap()).as_str();

  if args.opts.require_on_load {
    final_source += format!("\n(typeof module !== 'undefined' ? module : {{}}).exports = $esfuse$.require({});\n", utils::serialize_json(&args.locator.url, &args.locator.url).unwrap()).as_str();
  }

  final_source += format!("\n//# sourceMappingURL={}\n", ModuleLocator::new(
    args.locator.kind,
    format!("{}.map", &args.locator.specifier),
    args.locator.params,
  ).url).as_str();

  OnBundleResult {
    result: Ok(OnBundleResultData {
      entry: args.locator.url,
      mime_type: String::from("text/javascript"),

      code: final_source,
      map: final_source_map.to_json(None).expect("Should have been able to serialize the source map"),
    }),
    dependencies: vec![
    ],
  }
}

fn count_newlines(s: &str) -> usize {
  s.as_bytes().iter().filter(|&&c| c == b'\n').count()
}
