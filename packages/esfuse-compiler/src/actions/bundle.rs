extern crate queues;

use anyhow::{Context};
use parcel_resolver::{CacheCow, SpecifierType, Resolution};
use queues::*;
use std::{collections::{HashMap, HashSet}, borrow::Cow};

use crate::types::*;
use crate::{transforms::TransformError, utils};

pub struct BundleOutput {
  pub code: String,
  pub map: String,

  pub errors: HashMap<String, TransformError>,
  pub resolutions: HashMap<String, HashMap<String, String>>,
}

pub fn bundle(initial: ModuleLocator, project: &Project) -> anyhow::Result<BundleOutput> {
  let mut queued: Queue<ModuleLocator> = queue![initial];
  let mut traversed = HashSet::new();

  let mut errors: HashMap<String, TransformError> = HashMap::new();
  let mut resolutions: HashMap<String, HashMap<String, String>> = HashMap::new();

  let mut final_source = String::default();
  let mut final_source_map = parcel_sourcemap::SourceMap::new("");
  let mut final_nl_count = 0;

  let resolver_fs = parcel_resolver::OsFileSystem::default();
  let resolver_cache = parcel_resolver::Cache::new(resolver_fs);
  let resolver = parcel_resolver::Resolver::parcel(
    Cow::from(&project.root),
    CacheCow::Owned(resolver_cache),
  );

  while queued.size() > 0 {
    let locator = queued.remove()
      .expect("Should have been able to pop a value from the queue");

    if !traversed.contains(&locator) {
      traversed.insert(locator.clone());
    } else {
      continue
    }

    let transform_result
      = crate::transforms::transform(&locator.fetch(project)?, project);

    if let Err(err) = transform_result {
      errors.insert(locator.url(), err);
      continue
    }

    let transform_output = transform_result.unwrap();

    if let Some(map) = transform_output.map {
      let mut map_data = parcel_sourcemap::SourceMap::from_json("/", &map)
        .expect("Assertion failed: Expected the SWC-generated sourcemap to be readable");

      final_source_map.add_sourcemap(&mut map_data, final_nl_count)
        .expect("Assertion failed: Expected the SWC-generated sourcemap to be well-structured");
    }

    if traversed.len() > 1 {
      final_source += "\n";
      final_nl_count += 1;
    }

    final_source += &transform_output.code;
    final_nl_count += count_newlines(transform_output.code.as_str()) as i64;

    let module_from = locator.physical_path(project)
      .unwrap_or_default();

    for import in transform_output.imports {
      let specifier = match import.starts_with('/') {
        true => ModuleLocator::from_url(&import)?.physical_path(project).unwrap().to_string_lossy().to_string(),
        false => import.clone(),
      };

      let resolution = resolver.resolve(&specifier, &module_from, SpecifierType::Cjs).result
        .map_err(|error| TransformError::ResolutionError { error })
        .context(format!("Failed to resolve {} from {:?}", specifier, module_from))?;

      match resolution {
        (Resolution::Path(p), query) => {
          let dependency_locator = project.locator_from_path(
            &p,
            &utils::parse_query(query),
          );

          resolutions.entry(locator.url()).or_default()
            .insert(import, dependency_locator.url());

          queued.add(dependency_locator)
            .unwrap();
        }

        _ => {}
      }
    }
  }

  Ok(BundleOutput {
    code: final_source,
    map: final_source_map.to_json(None).expect("Should have been able to serialize the source map"),

    errors,
    resolutions,
  })
}

fn count_newlines(s: &str) -> usize {
  s.as_bytes().iter().filter(|&&c| c == b'\n').count()
}
