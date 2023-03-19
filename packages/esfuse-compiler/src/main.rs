use std::path::{PathBuf, Path};

use esfuse::types::*;
use esfuse::{bundle};

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

fn main() {
  let mut args = std::env::args();

  // Skip the program name
  args.next();

  let filename = args.next()
    .map(PathBuf::from)
    .expect("A file name must be provided")
    .canonicalize()
    .unwrap();

  let root = find_root(&filename).unwrap();
  let mut project = Project::new(root);

  project.register_ns("ylc", &project.root.join(".yarn/cache"));
  project.register_ns("ygc", &PathBuf::from("/Users/mael.nison/.yarn/berry/cache"));

  let entry_point = project.locator_from_path(
    &filename,
    &Default::default(),
  );

  let output = bundle(entry_point, &project).unwrap();
  print!("{}", output.code);
}
