mod difference;

use colored::Colorize;
use difference::Difference;
use serde::Deserialize;
use std::{collections::HashMap, fmt::Debug, fs::read_to_string, path::Path};

#[derive(Deserialize, Debug, Eq, PartialEq, Clone, Hash)]
pub struct Package {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
    #[serde(default)]
    dependencies: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PackageDiff {
    pub name: String,
    pub version: Difference<String>,
    pub source: Difference<String>,
    pub checksum: Difference<String>,
    pub dependencies: Vec<Difference<String>>,
}

impl PartialOrd for PackageDiff {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for PackageDiff {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PackageDiff {
    pub fn diff(a: Package, b: Package) -> PackageDiff {
        if a.name != b.name {
            panic!("diffing different packages is not supported");
        }
        PackageDiff {
            name: a.name,
            version: Difference::diff(a.version, b.version),
            source: Difference::diff_opt(a.source, b.source),
            checksum: Difference::diff_opt(a.checksum, b.checksum),
            dependencies: Difference::diff_vec(a.dependencies, b.dependencies),
        }
    }

    pub fn added(p: Package) -> PackageDiff {
        PackageDiff {
            name: p.name,
            version: Difference::Added(p.version),
            source: p
                .source
                .map_or(Difference::Empty, |source| Difference::Added(source)),
            checksum: p
                .checksum
                .map_or(Difference::Empty, |checksum| Difference::Added(checksum)),
            dependencies: p
                .dependencies
                .into_iter()
                .map(|dependency| Difference::Added(dependency))
                .collect(),
        }
    }

    pub fn removed(p: Package) -> PackageDiff {
        PackageDiff {
            name: p.name,
            version: Difference::Removed(p.version),
            source: p
                .source
                .map_or(Difference::Empty, |source| Difference::Removed(source)),
            checksum: p
                .checksum
                .map_or(Difference::Empty, |checksum| Difference::Removed(checksum)),
            dependencies: p
                .dependencies
                .into_iter()
                .map(|dependency| Difference::Removed(dependency))
                .collect(),
        }
    }

    pub fn is_equal_or_empty(&self) -> bool {
        self.version.is_equal()
            && (self.source.is_equal() || self.source.is_empty())
            && (self.checksum.is_equal() || self.checksum.is_empty())
            && self
                .dependencies
                .iter()
                .all(|dependency| dependency.is_equal() || dependency.is_empty())
    }

    fn pretty_print_version(&self) {
        match &self.version {
            Difference::Removed(version) => {
                println!("{}", format!("-version = \"{}\"", version).red());
            }
            Difference::Equal(version) => println!(" version = \"{}\"", version),
            Difference::Modified { old, new } => {
                println!("{}", format!("-version = \"{}\"", old).red());
                println!("{}", format!("+version = \"{}\"", new).green());
            }
            Difference::Added(version) => {
                println!("{}", format!("+version = \"{}\"", version).green())
            }
            _ => unreachable!("oh what have you done"),
        }
    }

    fn pretty_print_source(&self) {
        match &self.source {
            Difference::Removed(source) => {
                println!("{}", format!("-source = \"{}\"", source).red())
            }
            Difference::Equal(source) => println!(" source = \"{}\"", source),
            Difference::Modified { old, new } => {
                println!("{}", format!("-source = \"{}\"", old).red());
                println!("{}", format!("+source = \"{}\"", new).green());
            }
            Difference::Added(source) => {
                println!("{}", format!("+source = \"{}\"", source).green())
            }
            _ => {}
        }
    }

    fn pretty_print_checksum(&self) {
        match &self.checksum {
            Difference::Removed(checksum) => {
                println!("{}", format!("-checksum = \"{}\"", checksum).red())
            }
            Difference::Equal(checksum) => println!(" checksum = \"{}\"", checksum),
            Difference::Modified { old, new } => {
                println!("{}", format!("-checksum = \"{}\"", old).red());
                println!("{}", format!("+checksum = \"{}\"", new).green());
            }
            Difference::Added(checksum) => {
                println!("{}", format!("+checksum = \"{}\"", checksum).green())
            }
            _ => {}
        }
    }

    fn pretty_print_dependencies(&self, verbose: bool) {
        println!(" dependencies = [");
        for dependency in self.dependencies.iter() {
            match dependency {
                Difference::Removed(dependency) => {
                    println!("{}", format!("- \"{}\",", dependency).red())
                }
                Difference::Equal(dependency) => {
                    if verbose {
                        println!("  \"{}\",", dependency);
                    }
                }
                Difference::Modified { old, new } => {
                    println!("{}", format!("- \"{}\",", old).red());
                    println!("{}", format!("+ \"{}\",", new).green());
                }
                Difference::Added(dependency) => {
                    println!("{}", format!("+ \"{}\",", dependency).green())
                }
                _ => {}
            }
        }
        println!(" ]");
    }

    pub fn pretty_print_package(&self, verbose: bool) {
        println!(" [[package]]");
        println!(" name = \"{}\"", self.name);
        self.pretty_print_version();
        self.pretty_print_source();
        self.pretty_print_checksum();
        self.pretty_print_dependencies(verbose);
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct CargoLock {
    pub version: u8,
    pub package: Vec<Package>,
}

impl CargoLock {
    pub fn load_lock<P: AsRef<Path>>(path: P) -> Self {
        let contents = read_to_string(path).expect("reading should succeed");
        toml::from_str(&contents).expect("parsing should succeed")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CargoLockDiff {
    pub version: Difference<u8>,
    pub package: Vec<PackageDiff>,
}

impl CargoLockDiff {
    pub fn difference(a: CargoLock, b: CargoLock) -> Self {
        let version = Difference::diff(a.version, b.version);

        let a: HashMap<String, Package> = HashMap::from_iter(
            a.package
                .into_iter()
                .map(|package| (package.name.clone(), package)),
        );
        let b: HashMap<String, Package> = HashMap::from_iter(
            b.package
                .into_iter()
                .map(|package| (package.name.clone(), package)),
        );

        let mut package = Vec::with_capacity(a.len().max(b.len()));

        for (name, old_package) in a.iter() {
            if let Some(new_package) = b.get(name) {
                package.push(PackageDiff::diff(old_package.clone(), new_package.clone()));
            } else {
                package.push(PackageDiff::removed(old_package.clone()));
            }
        }

        for (name, new_package) in b.into_iter() {
            if a.contains_key(&name) {
                continue;
            }
            package.push(PackageDiff::added(new_package));
        }

        package.sort();

        Self { version, package }
    }

    fn pretty_print_version(&self) {
        match self.version {
            Difference::Equal(version) => println!(" version = {}", version),
            Difference::Modified { old, new } => {
                println!("{}", format!("-version = {}", old).red());
                println!("{}", format!("+version = {}", new).green());
            }
            _ => unreachable!("oh what have you done"),
        }
    }

    pub fn pretty_print(&self, verbose: bool) {
        self.pretty_print_version();
        if !self.package.is_empty() {
            println!();
        }

        for package in self.package[..self.package.len() - 1].iter() {
            if !package.is_equal_or_empty() {
                package.pretty_print_package(verbose);
                println!();
            }
        }

        if !self.package[self.package.len() - 1].is_equal_or_empty() {
            self.package[self.package.len() - 1].pretty_print_package(verbose);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn tokio_1_15_0_lock() -> Package {
        Package {
            name: "tokio".to_string(),
            version: "1.15.0".to_string(),
            source: Some("registry+https://github.com/rust-lang/crates.io-index".to_string()),
            checksum: Some(
                "fbbf1c778ec206785635ce8ad57fe52b3009ae9e0c9f574a728f3049d3e55838".to_string(),
            ),
            dependencies: vec![
                "bytes",
                "libc",
                "memchr",
                "mio",
                "num_cpus",
                "once_cell",
                "parking_lot",
                "pin-project-lite",
                "signal-hook-registry",
                "tokio-macros",
                "winapi",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }

    fn tokio_1_34_0_lock() -> Package {
        Package {
            name: "tokio".to_string(),
            version: "1.34.0".to_string(),
            source: Some("registry+https://github.com/rust-lang/crates.io-index".to_string()),
            checksum: Some(
                "d0c014766411e834f7af5b8f4cf46257aab4036ca95e9d2c144a10f59ad6f5b9".to_string(),
            ),
            dependencies: vec![
                "backtrace",
                "bytes",
                "libc",
                "mio",
                "num_cpus",
                "parking_lot",
                "pin-project-lite",
                "signal-hook-registry",
                "socket2",
                "tokio-macros",
                "windows-sys 0.48.0",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }

    #[test]
    fn test_package_diff() {
        let a = tokio_1_15_0_lock();
        let b = tokio_1_34_0_lock();
        let diff = PackageDiff::diff(a, b);
        let expected = PackageDiff {
            name: "tokio".to_string(),
            version: Difference::Modified {
                old: "1.15.0".to_string(),
                new: "1.34.0".to_string(),
            },
            source: Difference::Equal(
                "registry+https://github.com/rust-lang/crates.io-index".to_string(),
            ),
            checksum: Difference::Modified {
                old: "fbbf1c778ec206785635ce8ad57fe52b3009ae9e0c9f574a728f3049d3e55838".to_string(),
                new: "d0c014766411e834f7af5b8f4cf46257aab4036ca95e9d2c144a10f59ad6f5b9".to_string(),
            },
            dependencies: vec![
                Difference::Removed("memchr".to_string()),
                Difference::Removed("once_cell".to_string()),
                Difference::Removed("winapi".to_string()),
                Difference::Equal("bytes".to_string()),
                Difference::Equal("libc".to_string()),
                Difference::Equal("mio".to_string()),
                Difference::Equal("num_cpus".to_string()),
                Difference::Equal("parking_lot".to_string()),
                Difference::Equal("pin-project-lite".to_string()),
                Difference::Equal("signal-hook-registry".to_string()),
                Difference::Equal("tokio-macros".to_string()),
                Difference::Added("backtrace".to_string()),
                Difference::Added("socket2".to_string()),
                Difference::Added("windows-sys 0.48.0".to_string()),
            ],
        };

        assert_eq!(diff, expected);
    }

    #[test]
    fn test_cargo_lock_diff() {
        let a = CargoLock {
            version: 3,
            package: vec![tokio_1_15_0_lock()],
        };

        let b = CargoLock {
            version: 3,
            package: vec![tokio_1_34_0_lock()],
        };

        let diff = CargoLockDiff::difference(a, b);
        let expected = CargoLockDiff {
            version: Difference::Equal(3),
            package: vec![PackageDiff {
                name: "tokio".to_string(),
                version: Difference::Modified {
                    old: "1.15.0".to_string(),
                    new: "1.34.0".to_string(),
                },
                source: Difference::Equal(
                    "registry+https://github.com/rust-lang/crates.io-index".to_string(),
                ),
                checksum: Difference::Modified {
                    old: "fbbf1c778ec206785635ce8ad57fe52b3009ae9e0c9f574a728f3049d3e55838"
                        .to_string(),
                    new: "d0c014766411e834f7af5b8f4cf46257aab4036ca95e9d2c144a10f59ad6f5b9"
                        .to_string(),
                },
                dependencies: vec![
                    Difference::Removed("memchr".to_string()),
                    Difference::Removed("once_cell".to_string()),
                    Difference::Removed("winapi".to_string()),
                    Difference::Equal("bytes".to_string()),
                    Difference::Equal("libc".to_string()),
                    Difference::Equal("mio".to_string()),
                    Difference::Equal("num_cpus".to_string()),
                    Difference::Equal("parking_lot".to_string()),
                    Difference::Equal("pin-project-lite".to_string()),
                    Difference::Equal("signal-hook-registry".to_string()),
                    Difference::Equal("tokio-macros".to_string()),
                    Difference::Added("backtrace".to_string()),
                    Difference::Added("socket2".to_string()),
                    Difference::Added("windows-sys 0.48.0".to_string()),
                ],
            }],
        };

        assert_eq!(diff, expected);
    }
}
