use parcel_resolver::{SpecifierType, ResolverError};

use crate::types::*;
use crate::utils;
use crate::{CompilationError, Project};

pub async fn resolve(project: &Project, args: OnResolveArgs) -> OnResolveResult {
  if let Some(locator) = project.locator(&args.request) {
    return OnResolveResult {
      result: Ok(locator),
    }
  }

  if let Some(hook) = Project::resolve_plugin_hook(&project.on_resolve, &args.request) {
    return OnResolveResult {
      result: (hook.cb)(hook.data.clone(), args).await,
    };
  }

  let base = args.issuer.and_then(|locator| {
    locator.physical_path(project)
  }).unwrap_or_else(|| project.root.to_path_buf());

  let r =
    project.resolver.resolve(&args.request, &base, SpecifierType::Cjs);

  OnResolveResult {
    result: match r.result {
      Ok((parcel_resolver::Resolution::Path(p), query)) => {
        let mut params
          = query.map_or(Default::default(), |s| utils::parse_query(s.as_str()));

        params.append(&mut args.opts.force_params.clone());

        Ok(project.locator_from_path(&p, &params))
      }

      Err(err) => {
        Err(CompilationError {
          diagnostics: vec![
            utils::errors::Diagnostic::from_string_with_span(match err {
              ResolverError::UnknownScheme{..}
                => String::from("Unknown scheme"),
              ResolverError::UnknownError{..}
                => String::from("Unknown error"),
              ResolverError::FileNotFound { relative, from }
                => format!("File not found ({:?}, from {:?})", relative, from),
              ResolverError::ModuleNotFound { module }
                => format!("Module not found ({:?})", module),
              ResolverError::ModuleEntryNotFound { .. }
                => format!("Module entry not found"),
              ResolverError::ModuleSubpathNotFound { .. }
                => format!("Module subpath not found"),
              ResolverError::JsonError(_)
                => format!("Json error"),
              ResolverError::IOError(_)
                => format!("IO error"),
              ResolverError::PackageJsonError { path, .. }
                => format!("Invalid package manifest file ({:?})", &path),
              ResolverError::PackageJsonNotFound { from }
                => format!("Package manifest not found ({:?})", &from),
              ResolverError::InvalidSpecifier(_)
                => format!("Invalid specifier"),
              ResolverError::TsConfigExtendsNotFound { .. }
                => format!("Extended TypeScript configuration file not found"),
              ResolverError::PnpResolutionError(err)
                => err.to_string(),
            }, args.span)
          ].to_vec(),
        })
      }

      _ => {
        Err(CompilationError {
          diagnostics: vec![
            utils::errors::Diagnostic::from_string_with_span(String::from("Unsupported resolution"), args.span)
          ].to_vec(),
        })
      }
    },
  }
}
