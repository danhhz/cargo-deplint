// Copyright 2023 Daniel Harrison. All Rights Reserved.

use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CargoLock {
    pub package: Option<Vec<Package>>,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Lints {
    pub deny: Option<Vec<Deny>>,
}

#[derive(Debug, Deserialize)]
pub struct Deny {
    pub name: String,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug)]
enum Dep<'a> {
    Direct,
    // TODO: Store all indirect paths.
    Indirect(Vec<&'a str>),
}

pub fn run_lints(cargo_lock: &CargoLock, lints: &Lints) -> Result<Vec<String>, String> {
    let all_deps = build_dep_graph(cargo_lock);

    let mut ret = Vec::new();
    let Some(denies) = lints.deny.as_ref() else {
        return Ok(ret);
    };
    for deny in denies.iter() {
        let deps = all_deps
            .get(deny.name.as_str())
            .ok_or_else(|| format!("unknown crate in lints: {}", deny.name))?;
        let Some(deny_deps) = deny.dependencies.as_ref() else {
            continue;
        };
        for deny_dep in deny_deps.iter() {
            let _ = all_deps
                .get(deny_dep.as_str())
                .ok_or_else(|| format!("unknown crate in lints: {}", deny_dep))?;
            if let Some(path) = deps.deps.get(deny_dep.as_str()) {
                match path {
                    Dep::Direct => ret.push(format!("deny: {} -> {}", deny.name, deny_dep)),
                    Dep::Indirect(path) => {
                        assert_eq!(path.is_empty(), false);
                        ret.push(format!(
                            "deny: {} -> {} -> {}",
                            deny.name,
                            path.join(" -> "),
                            deny_dep
                        ))
                    }
                }
            }
        }
    }
    Ok(ret)
}

type DepGraph<'a> = BTreeMap<&'a str, PackageDeps<'a>>;

#[derive(Debug, Default)]
struct PackageDeps<'a> {
    work: Vec<&'a str>,
    deps: BTreeMap<&'a str, Dep<'a>>,
}

fn build_dep_graph<'a>(cargo_lock: &'a CargoLock) -> DepGraph<'a> {
    let mut all = DepGraph::new();

    // Insert all the direct dependencies.
    let Some(packages) = cargo_lock.package.as_ref() else {
        return all;
    };
    for package in packages.iter() {
        let direct = all.entry(package.name.as_str()).or_default();
        let Some(package_deps) = package.dependencies.as_ref() else {
            continue;
        };
        for package_dep in package_deps.iter() {
            direct.work.push(package_dep.as_str());
            direct.deps.insert(package_dep.as_str(), Dep::Direct);
        }
    }

    // Propagate transitive deps.
    for package in packages.iter() {
        fill_transitive(&mut all, package.name.as_str());
    }

    all
}

fn fill_transitive<'a>(all: &mut DepGraph<'a>, key: &'a str) {
    while let Some(work) = all.get_mut(key).and_then(|x| x.work.pop()) {
        fill_transitive(all, work);

        let key_deps = all.get(key).unwrap();
        let Some(work_deps) = all.get(work) else {
            continue;
        };
        let mut additions = Vec::new();
        for (transitive, x) in work_deps.deps.iter() {
            if !key_deps.deps.contains_key(*transitive) {
                let mut path = vec![work];
                match x {
                    Dep::Direct => {}
                    Dep::Indirect(p) => path.extend(p),
                }
                additions.push((*transitive, Dep::Indirect(path)));
            }
        }
        if !additions.is_empty() {
            let key_deps = all.get_mut(key).unwrap();
            for (k, v) in additions {
                key_deps.deps.insert(k, v);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deps() {
        datadriven::walk("tests/testdata", |tf| {
            let mut cargo_lock = String::new();
            tf.run(|tc| -> String {
                match tc.directive.as_str() {
                    "cargo_lock" => {
                        cargo_lock = tc.input.clone();
                        "ok\n".into()
                    }
                    "lints" => {
                        let cargo_lock: CargoLock = toml::from_str(&cargo_lock).unwrap();
                        let lints: Lints = toml::from_str(&tc.input).unwrap();
                        let res = run_lints(&cargo_lock, &lints);
                        match res {
                            Ok(violations) if violations.is_empty() => "ok\n".into(),
                            Ok(violations) => {
                                violations.into_iter().map(|x| format!("{}\n", x)).collect()
                            }
                            Err(err) => format!("error: {}\n", err),
                        }
                    }
                    x => panic!("unknown directive [{}]", x),
                }
            })
        });
    }
}
