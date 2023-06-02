use arca::ToArcaPath;
use parcel_resolver::{SpecifierType, ResolverError};

use crate::types::*;
use crate::utils;
use crate::{CompilationError, Project};

pub async fn resolve(project: &Project, args: OnResolveArgs) -> OnResolveResult {
  if let Some(locator) = project.locator(&args.request) {
    return OnResolveResult {
      result: Ok(OnResolveResultData { locator }),
      dependencies: vec![],
    }
  }

  Project::resolve_plugin_hook(
    &project.on_resolve,
    &args.request.clone(),
    args.clone(),
  ).await.unwrap_or_else(|| {
    resolve_no_hooks(project, args)
  })
}

pub fn resolve_no_hooks(project: &Project, args: OnResolveArgs) -> OnResolveResult {
  if args.request.as_str() == "esfuse/context" {
    let builtin_locator = ModuleLocator::new(
      ModuleLocatorKind::External,
      args.request,
      vec![],
    );

    return OnResolveResult {
      result: Ok(OnResolveResultData { locator: builtin_locator }),
      dependencies: vec![],
    }
  }

  if let Some(locator) = project.locator(&args.request) {
    return OnResolveResult {
      result: Ok(OnResolveResultData { locator }),
      dependencies: vec![],
    }
  }

  let base = args.issuer.clone().and_then(|locator| {
    locator.physical_path(project)
  }).unwrap_or_else(|| project.root.as_ref().clone());

  let (specifier, mut request_params) = args.request.split_once('?')
    .map(|(specifier, qs)| (specifier, utils::parse_query(qs)))
    .unwrap_or((&args.request, vec![]));

  let mut params = vec![];
  params.append(&mut args.opts.force_params.clone());
  params.append(&mut request_params);

  let r =
    project.resolver.resolve(&specifier, &base.to_path_buf(), SpecifierType::Cjs);

  OnResolveResult {
    result: match r.result {
      Ok((parcel_resolver::Resolution::Path(p), _)) => {

        Ok(OnResolveResultData {
          locator: project.locator_from_path(&p.to_arca(), &params).unwrap(),
        })
      },

      Ok((parcel_resolver::Resolution::Builtin(name), _)) => {
        Ok(OnResolveResultData {
          locator: ModuleLocator {
            url: name.clone(),
            kind: ModuleLocatorKind::External,
            specifier: name,
            params: vec![],
          },
        })
      },

      Err(err) => {
        Err(CompilationError {
          diagnostics: vec![
            utils::errors::Diagnostic::from_string_with_highlight(match err {
              ResolverError::UnknownScheme{..}
                => String::from("Unknown scheme"),
              ResolverError::UnknownError{..}
                => String::from("Unknown error"),
              ResolverError::FileNotFound { relative, from }
                => format!("File not found ({:?}, from {:?})", relative, from),
              ResolverError::ModuleNotFound { module }
                => format!("Module not found ({:?})", module),
              ResolverError::ModuleEntryNotFound { .. }
                => "Module entry not found".to_string(),
              ResolverError::ModuleSubpathNotFound { .. }
                => "Module subpath not found".to_string(),
              ResolverError::JsonError(_)
                => "Json error".to_string(),
              ResolverError::IOError(_)
                => "IO error".to_string(),
              ResolverError::PackageJsonError { path, error, .. }
                => format!("Invalid package manifest file ({:?} in {:?})", &error, &path),
              ResolverError::PackageJsonNotFound { from }
                => format!("Package manifest not found ({:?})", &from),
              ResolverError::InvalidSpecifier(_)
                => "Invalid specifier".to_string(),
              ResolverError::TsConfigExtendsNotFound { .. }
                => "Extended TypeScript configuration file not found".to_string(),
              ResolverError::PnpResolutionError(err)
                => err.to_string(),
            }, utils::errors::Highlight {
              source: args.issuer.map(|locator| locator.url),
              subject: None,
              label: None,
              span: args.span,
            }),
          ],
        })
      },

      unknown_resolution => {
        Err(CompilationError {
          diagnostics: vec![
            utils::errors::Diagnostic::from_string_with_highlight(
              format!("Unsupported resolution: {:?}", unknown_resolution),
              utils::errors::Highlight {
                source: args.issuer.map(|locator| locator.url.clone()),
                subject: None,
                label: None,
                span: args.span,
              },
            ),
          ],
        })
      },
    },

    dependencies: vec![],
  }
}
