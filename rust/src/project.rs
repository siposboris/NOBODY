//! Experimental multi-file Nobody project linker: one syntax language per file.
//!
//! This is intentionally a bounded, deterministic *project build* layer,
//! not source-level imports, name-spaced modules, or a package manager.
use crate::syntax::Statement;
use crate::{parse_validated, same_ir, translate_source, Diagnostic};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path};

const MAX_MANIFEST: usize = 65_536;
const MAX_SOURCE: usize = 1_048_576;
const MAX_MODULES: usize = 64;

#[derive(Clone, Debug)]
struct ModuleSpec {
    profile: String,
    path: String,
    dependencies: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Project {
    /// The complete validated program in canonical 'ai' syntax.
    pub source: String,
    /// Dependency-first order of source files included in the program.
    pub modules: Vec<String>,
}

fn issue(detail: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code: 5003,
        line: 0,
        column: 0,
        detail: detail.into(),
        repair: "Use a valid nobody.project.v1 manifest with acyclic, local module paths, unique names and explicit dependencies.",
    }
}

fn read_capped(path: &Path, limit: usize) -> Result<String, Diagnostic> {
    let file = File::open(path).map_err(|error| issue(format!("Cannot open {path:?}: {error}")))?;
    let mut bytes = Vec::new();
    file.take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| issue(format!("Cannot read {path:?}: {error}")))?;
    if bytes.len() > limit {
        return Err(issue(format!(
            "Project input exceeds its {limit}-byte limit"
        )));
    }
    String::from_utf8(bytes).map_err(|_| issue("Project manifest or source is not UTF-8"))
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.as_bytes()[0].is_ascii_lowercase()
        && name
            .bytes()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == b'_')
}

fn visit(
    name: &str,
    specs: &BTreeMap<String, ModuleSpec>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    order: &mut Vec<String>,
) -> Result<(), Diagnostic> {
    if visited.contains(name) {
        return Ok(());
    }
    let spec = specs
        .get(name)
        .ok_or_else(|| issue(format!("Dependency {name:?} is not declared")))?;
    if !visiting.insert(name.to_owned()) {
        return Err(issue(format!(
            "Cyclic project dependency involving {name:?}"
        )));
    }
    for dependency in &spec.dependencies {
        visit(dependency, specs, visiting, visited, order)?;
    }
    visiting.remove(name);
    visited.insert(name.to_owned());
    order.push(name.to_owned());
    Ok(())
}

/// Load dependency-ordered Nobody source files and validate the merged AST.
/// Each file is parsed using its own selected profile before translation into
/// the shared 'ai' syntax; accidental keyword mixing is not permitted.
///
/// Format (whitespace-separated, no spaces in file paths):
/// nobody.project.v1
/// module math sk math.nbd -
/// module main en main.nbd math
/// entry main
pub fn load_project(path: &Path) -> Result<Project, Diagnostic> {
    let manifest = fs::canonicalize(path)
        .map_err(|error| issue(format!("Cannot locate project manifest {path:?}: {error}")))?;
    let root = manifest
        .parent()
        .ok_or_else(|| issue("Project manifest has no parent directory"))?;
    let text = read_capped(&manifest, MAX_MANIFEST)?;
    let mut specs = BTreeMap::<String, ModuleSpec>::new();
    let mut entry = None;
    let mut version = false;
    for (line_number, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !version {
            if line != "nobody.project.v1" {
                return Err(issue("First manifest directive must be nobody.project.v1"));
            }
            version = true;
            continue;
        }
        let parts: Vec<_> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["module", name, profile, filename, dependencies] => {
                if specs.len() >= MAX_MODULES {
                    return Err(issue("Project exceeds the 64-module limit"));
                }
                if !valid_name(name) {
                    return Err(issue(format!(
                        "Invalid lowercase module name on line {}",
                        line_number + 1
                    )));
                }
                if crate::profiles::profile(profile).is_none() {
                    return Err(issue(format!(
                        "Unknown syntax profile {profile:?} on line {}",
                        line_number + 1
                    )));
                }
                let relative = Path::new(filename);
                if relative.extension().is_none_or(|ext| ext != "nbd")
                    || !relative
                        .components()
                        .all(|part| matches!(part, Component::Normal(_)))
                {
                    return Err(issue(format!(
                        "Unsafe or non-.nbd module path on line {}",
                        line_number + 1
                    )));
                }
                let dependencies = if *dependencies == "-" {
                    Vec::new()
                } else {
                    dependencies
                        .split(',')
                        .map(str::to_owned)
                        .collect::<Vec<_>>()
                };
                let mut unique = BTreeSet::new();
                if dependencies
                    .iter()
                    .any(|dependency| !valid_name(dependency) || !unique.insert(dependency.clone()))
                {
                    return Err(issue(format!(
                        "Invalid or duplicate dependency on line {}",
                        line_number + 1
                    )));
                }
                if specs
                    .insert(
                        (*name).to_owned(),
                        ModuleSpec {
                            profile: (*profile).to_owned(),
                            path: (*filename).to_owned(),
                            dependencies,
                        },
                    )
                    .is_some()
                {
                    return Err(issue(format!("Duplicate module name {name:?}")));
                }
            }
            ["entry", name] if entry.is_none() && valid_name(name) => {
                entry = Some((*name).to_owned());
            }
            _ => {
                return Err(issue(format!(
                    "Invalid project directive on line {}",
                    line_number + 1
                )))
            }
        }
    }
    if !version {
        return Err(issue("Project manifest has no version header"));
    }
    let entry = entry.ok_or_else(|| issue("Project requires exactly one entry module"))?;
    if specs.is_empty() {
        return Err(issue("Project has no modules"));
    }
    let mut order = Vec::new();
    visit(
        &entry,
        &specs,
        &mut BTreeSet::new(),
        &mut BTreeSet::new(),
        &mut order,
    )?;
    if order.len() != specs.len() {
        return Err(issue(
            "Project declares unreachable modules; connect them through explicit dependencies",
        ));
    }

    let mut merged = String::new();
    let mut exports = BTreeMap::<String, String>::new();
    let mut canonical_paths = BTreeSet::new();
    for name in &order {
        let spec = &specs[name];
        let resolved = fs::canonicalize(root.join(&spec.path))
            .map_err(|error| issue(format!("Cannot resolve module {name:?}: {error}")))?;
        if !resolved.starts_with(root) || !resolved.is_file() {
            return Err(issue(format!(
                "Module {name:?} resolves outside the project root or is not a file"
            )));
        }
        if !canonical_paths.insert(resolved.clone()) {
            return Err(issue("Two modules resolve to the same source file"));
        }
        let source = read_capped(&resolved, MAX_SOURCE)?;
        parse_validated(&source, &spec.profile)?;
        let translated = translate_source(&source, &spec.profile, "ai")?;
        if !same_ir(&source, &spec.profile, &translated, "ai")? {
            return Err(issue(format!(
                "Module {name:?} changes its canonical IR during profile translation"
            )));
        }
        let parsed = parse_validated(&translated, "ai")?;
        for stmt in &parsed.statements {
            let symbol = match stmt {
                Statement::Function { name, .. } | Statement::Let { name, .. } => Some(name),
                _ => None,
            };
            if let Some(symbol) = symbol {
                if let Some(previous) = exports.insert(symbol.clone(), name.clone()) {
                    return Err(issue(format!(
                        "Top-level name {symbol:?} is declared in both {previous:?} and {name:?}"
                    )));
                }
            }
        }
        if merged.len().saturating_add(translated.len() + 2) > MAX_SOURCE {
            return Err(issue("Combined project source exceeds 1 MiB"));
        }
        merged.push_str(&translated);
        merged.push('\n');
    }
    parse_validated(&merged, "ai")?;
    Ok(Project {
        source: merged,
        modules: order,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_manifest_and_unknown_header() {
        let missing = Path::new("/nonexistent/nobody_project_missing_manifest.nbproj");
        assert_eq!(load_project(missing).unwrap_err().code, 5003);
        assert!(!valid_name("BadName"));
        assert!(!valid_name("_internal"));
        assert!(valid_name("module_1"));
    }

    #[test]
    fn dependency_cycles_and_undeclared_nodes_are_rejected() {
        let mut specs = BTreeMap::new();
        specs.insert(
            "one".into(),
            ModuleSpec {
                profile: "ai".into(),
                path: "one.nbd".into(),
                dependencies: vec!["two".into()],
            },
        );
        specs.insert(
            "two".into(),
            ModuleSpec {
                profile: "ai".into(),
                path: "two.nbd".into(),
                dependencies: vec!["one".into()],
            },
        );
        assert_eq!(
            visit(
                "one",
                &specs,
                &mut BTreeSet::new(),
                &mut BTreeSet::new(),
                &mut Vec::new()
            )
            .unwrap_err()
            .code,
            5003
        );
        specs.get_mut("two").unwrap().dependencies = vec!["missing".into()];
        assert_eq!(
            visit(
                "one",
                &specs,
                &mut BTreeSet::new(),
                &mut BTreeSet::new(),
                &mut Vec::new()
            )
            .unwrap_err()
            .code,
            5003
        );
    }
}
