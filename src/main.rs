use std::{
    collections::{BTreeMap, HashSet},
    env,
    fmt::{self, Debug, Formatter},
    fs::read_to_string,
    path::{Path, PathBuf},
};

use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct CargoLock {
    version: u8,
    package: Vec<Package>,
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
struct Package {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
    #[serde(default)]
    dependencies: Vec<String>,
}

#[derive(Default)]
struct PackageDiff {
    version: Option<(String, String)>,
    source: Option<(Option<String>, Option<String>)>,
    checksum: Option<(Option<String>, Option<String>)>,
    dependencies: Option<(Vec<String>, Vec<String>)>,
}

impl PackageDiff {
    fn new() -> Self {
        Default::default()
    }

    fn diff(a: Package, b: Package) -> Option<PackageDiff> {
        let mut package_diff = PackageDiff::new();

        if a.version != b.version {
            package_diff.version = Some((a.version, b.version));
        }

        if a.source != b.source {
            package_diff.source = Some((a.source, b.source));
        }

        if a.checksum != b.checksum {
            package_diff.checksum = Some((a.checksum, b.checksum));
        }

        let a_deps: HashSet<&String> = HashSet::from_iter(a.dependencies.iter());
        let b_deps: HashSet<&String> = HashSet::from_iter(b.dependencies.iter());
        let common_deps: HashSet<&String> = a_deps.intersection(&b_deps).cloned().collect();

        // YUCK: The double clone
        let a_diff = a_deps
            .difference(&common_deps)
            .cloned()
            .cloned()
            .collect::<Vec<_>>();

        let b_diff = b_deps
            .difference(&common_deps)
            .cloned()
            .cloned()
            .collect::<Vec<_>>();

        if !a_diff.is_empty() || !b_diff.is_empty() {
            package_diff.dependencies = Some((a_diff, b_diff))
        }

        if package_diff.is_empty() {
            None
        } else {
            Some(package_diff)
        }
    }

    fn is_empty(&self) -> bool {
        self.version.is_none()
            && self.source.is_none()
            && self.checksum.is_none()
            && self.dependencies.is_none()
    }
}

impl Debug for PackageDiff {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("PackageDiff");
        if let Some(version) = &self.version {
            debug_struct.field("version", version);
        }
        if let Some(source) = &self.source {
            debug_struct.field("source", source);
        }
        if let Some(checksum) = &self.checksum {
            if self.version.is_none() {
                debug_struct.field("checksum", checksum);
            }
        }
        if let Some(dependencies) = &self.dependencies {
            debug_struct.field("dependencies", dependencies);
        }
        debug_struct.finish()
    }
}

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    target: PathBuf,
}

fn load_lock<P: AsRef<Path>>(path: P) -> CargoLock {
    let contents = read_to_string(path).expect("reading should succeed");
    toml::from_str(&contents).expect("parsing should succeed")
}

fn main() {
    let lockfile_path = env::current_dir()
        .expect("the current directory should be valid")
        .join("Cargo.lock");
    let local_lock = load_lock(lockfile_path);

    let cli = Cli::parse();
    let target_lock = load_lock(cli.target);

    if local_lock.version != target_lock.version {
        panic!("I don't know how to handle this situation")
    }

    let lockfile_packages = BTreeMap::from_iter(local_lock.package.iter().map(|p| (&p.name, p)));
    let target_packages = BTreeMap::from_iter(target_lock.package.iter().map(|p| (&p.name, p)));

    let mut seen = HashSet::new();
    for (package_name, package) in lockfile_packages.iter() {
        seen.insert(*package_name);
        if let Some(target_package) = target_packages.get(*package_name) {
            if let Some(diff) = PackageDiff::diff((**package).clone(), (**target_package).clone()) {
                println!("{} {:?}", package_name, diff);
            }
        }
    }

    for (package_name, package) in target_packages.iter() {
        if !seen.insert(*&package_name) {
            continue;
        }
        if let Some(target_package) = lockfile_packages.get(*package_name) {
            if let Some(diff) = PackageDiff::diff((**target_package).clone(), (**package).clone()) {
                println!("{} {:?}", package_name, diff);
            }
        }
    }
}
