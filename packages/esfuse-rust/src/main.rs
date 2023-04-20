use std::{path::{PathBuf, Path}, sync::Arc};

fn find_root(p: &Path) -> Option<PathBuf> {
  let lockfile_path = p.join("yarn.lock");

  if lockfile_path.exists() {
    Some(p.to_path_buf())
  } else if let Some(directory_path) = p.parent() {
    find_root(directory_path)
  } else {
    None
  }
}

#[tokio::main]
async fn main() {
  let mut args = std::env::args();

  // Skip the program name
  args.next();

  let filename = args.next()
    .map(PathBuf::from)
    .expect("A file name must be provided")
    .canonicalize()
    .unwrap();

  let root = find_root(&filename).unwrap();
  let mut project = esfuse::Project::new(&root);

  project.register_ns("ylc", &project.root.join(".yarn/cache"));
  project.register_ns("ygc", &PathBuf::from("/Users/mael.nison/.yarn/berry/cache"));

  let entry_point = project.locator_from_path(
    &filename,
    &vec![],
  ).unwrap();

  let project_arc = Arc::new(project);
  let bundle = esfuse::actions::bundle::bundle(project_arc, esfuse::types::OnBundleArgs {
    locator: entry_point,
    opts: Default::default(),
  }).await.result.unwrap();

  print!("{}", bundle.code);
}
