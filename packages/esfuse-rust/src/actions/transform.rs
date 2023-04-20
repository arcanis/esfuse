use crate::types::*;
use crate::Project;

use super::fetch::{fetch_no_hooks, fetch};

pub fn transform_no_hooks(project: &Project, args: OnTransformArgs) -> OnTransformResult {
  let fetch_res
    = fetch_no_hooks(project, OnFetchArgs {
        locator: args.locator.clone(),
        opts: OnFetchOpts {
          user_data: args.opts.user_data.clone(),
        },
      });

  match fetch_res.result {
    Ok(fetch_data) => {
      crate::transforms::transform(project, fetch_data, args)
    },

    Err(err) => {
      OnTransformResult {
        result: Err(err),
        dependencies: fetch_res.dependencies,
      }
    },
  }
}

pub async fn transform(project: &Project, args: OnTransformArgs) -> OnTransformResult {
  let fetch_res
    = fetch(project, OnFetchArgs {
        locator: args.locator.clone(),
        opts: OnFetchOpts {
          user_data: args.opts.user_data.clone(),
        },
      }).await;

  match fetch_res.result {
    Ok(fetch_data) => {
      crate::transforms::transform(project, fetch_data, args)
    },

    Err(err) => {
      OnTransformResult {
        result: Err(err),
        dependencies: fetch_res.dependencies,
      }
    },
  }
}
