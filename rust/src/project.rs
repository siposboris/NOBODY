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

/// A source file must not silently interpret another syntax profile's keyword
/// as an undeclared identifier. Identifiers explicitly declared in the same
/// file remain legitimate user names (even if they coincide with a foreign
/// word); comments and string literals are not inspected.
fn collect_declarations(statements: &[Statement], names: &mut BTreeSet<String>) {
    for statement in statements {
        match statement {
            Statement::Let { name, .. } => {
                names.insert(name.clone());
            }
            Statement::Function {
                name,
                parameters,
                body,
                ..
            } => {
                names.insert(name.clone());
                names.extend(parameters.iter().cloned());
                collect_declarations(body, names);
            }
            Statement::For { variable, body, .. } => {
                names.insert(variable.clone());
                collect_declarations(body, names);
            }
            Statement::While { body, .. } => collect_declarations(body, names),
            Statement::If {
                branches,
                otherwise,
            } => {
                for (_, body) in branches {
                    collect_declarations(body, names);
                }
                if let Some(body) = otherwise {
                    collect_declarations(body, names);
                }
            }
            _ => {}
        }
    }
}

fn reject_foreign_keywords(
    source: &str,
    profile: &str,
    parsed: &[Statement],
) -> Result<(), Diagnostic> {
    use crate::{profiles, TokenKind};
    let mut declared = BTreeSet::new();
    collect_declarations(parsed, &mut declared);
    for token in crate::scan(source)? {
        if token.kind != TokenKind::Name
            || declared.contains(&token.value)
            || profiles::semantic(profile, &token.value).is_some()
            || (profile == "en" && token.value == "def")
            || (profile == "sk" && token.value == "kym")
        {
            continue;
        }
        if let Some(other) = profiles::PROFILES.iter().find(|other| {
            other.code != profile && profiles::semantic(other.code, &token.value).is_some()
        }) {
            return Err(Diagnostic {
                code: 3004,
                line: token.line,
                column: token.column,
                detail: format!(
                    "Foreign syntax keyword {:?} belongs to profile {:?}, not {:?}",
                    token.value, other.code, profile
                ),
                repair: "Use only the selected syntax profile's keywords in this source file, or explicitly declare a user identifier with that spelling.",
            });
        }
    }
    Ok(())
}

/// Parse actual, localized source-level import directives at file scope.
///
/// A v2 source starts with zero or more localized import directives,
/// separated by comments and blank lines. These are compile-time declarations;
/// they do not become runtime expressions. Use lexer tokens rather than
/// naive line matching, so strings never become accidental imports.
fn source_imports(
    source: &str,
    profile: &str,
    enabled: bool,
) -> Result<(String, BTreeSet<String>), Diagnostic> {
    use crate::TokenKind;
    let keyword = crate::profiles::word(profile, "import")
        .ok_or_else(|| issue(format!("Missing import keyword in profile {profile:?}")))?;
    let tokens = crate::scan(source)?;
    let mut by_line: BTreeMap<usize, Vec<&crate::Token>> = BTreeMap::new();
    for token in &tokens {
        by_line.entry(token.line).or_default().push(token);
    }
    let mut output = String::with_capacity(source.len());
    let mut imports = BTreeSet::new();
    let mut started_body = false;
    for (index, line) in source.split_inclusive('\n').enumerate() {
        let line_tokens = by_line.get(&(index + 1));
        let meaningful = line_tokens.and_then(|tokens| tokens.first());
        let is_import =
            meaningful.is_some_and(|token| token.kind == TokenKind::Name && token.value == keyword);
        if !is_import {
            if meaningful.is_some_and(|token| token.kind != TokenKind::Comment) {
                started_body = true;
            }
            output.push_str(line);
            continue;
        }
        if !enabled {
            return Err(issue(
                "Language-level imports require nobody.project.v2 (not v1)",
            ));
        }
        if started_body {
            return Err(issue(format!(
                "Import directive must precede all executable code (line {})",
                index + 1
            )));
        }
        let tokens = line_tokens.expect("a source import token exists");
        let valid_shape = matches!(
            tokens.as_slice(),
            [_, module]
                if module.kind == TokenKind::Name
        ) || matches!(
            tokens.as_slice(),
            [_, module, comment]
                if module.kind == TokenKind::Name && comment.kind == TokenKind::Comment
        );
        if !valid_shape {
            return Err(issue(format!(
                "Invalid import directive on line {}",
                index + 1
            )));
        }
        let module = &tokens[1].value;
        if !valid_name(module) || crate::profiles::is_reserved(profile, module) {
            return Err(issue(format!(
                "Invalid or reserved imported module name {module:?} on line {}",
                index + 1
            )));
        }
        if !imports.insert(module.clone()) {
            return Err(issue(format!("Duplicate source import {module:?}")));
        }
        // Preserve line numbering in downstream parser diagnostics.
        if line.ends_with('\n') {
            output.push('\n');
        }
    }
    Ok((output, imports))
}

/// In a v2 project, a symbol exported by another file can be referenced
/// only through a directly declared import of that file's module name.
/// This is an intentionally conservative whole-file lexical check; full
/// scope-aware namespaces and object-file linking are future work.
fn check_direct_import_visibility(
    module: &str,
    source: &str,
    statements: &[Statement],
    imported: &BTreeSet<String>,
    owners: &BTreeMap<String, String>,
) -> Result<(), Diagnostic> {
    use crate::TokenKind;
    let mut locally_declared = BTreeSet::new();
    collect_declarations(statements, &mut locally_declared);
    for token in crate::scan(source)? {
        if token.kind != TokenKind::Name || locally_declared.contains(&token.value) {
            continue;
        }
        if let Some(owner) = owners.get(&token.value) {
            if owner != module && !imported.contains(owner) {
                return Err(issue(format!(
                    "Module {module:?} uses symbol {:?} from {owner:?} without a direct import",
                    token.value
                )));
            }
        }
    }
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
    let mut version = 0u8;
    for (line_number, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if version == 0 {
            version =
                match line {
                    "nobody.project.v1" => 1,
                    "nobody.project.v2" => 2,
                    _ => return Err(issue(
                        "First manifest directive must be nobody.project.v1 or nobody.project.v2",
                    )),
                };
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
    if version == 0 {
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
    let mut visible = Vec::new();
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
        let full_source = read_capped(&resolved, MAX_SOURCE)?;
        let (source, imports) = source_imports(&full_source, &spec.profile, version == 2)?;
        if version == 2 && imports != spec.dependencies.iter().cloned().collect() {
            return Err(issue(format!("Source import declarations for module {name:?} must match its manifest dependencies exactly")));
        }
        let original = parse_validated(&source, &spec.profile)?;
        reject_foreign_keywords(&source, &spec.profile, &original.statements)?;
        let translated = translate_source(&source, &spec.profile, "ai")?;
        if !same_ir(&source, &spec.profile, &translated, "ai")? {
            return Err(issue(format!(
                "Module {name:?} changes its canonical IR during profile translation"
            )));
        }
        let parsed = parse_validated(&translated, "ai")?;
        if version == 2 {
            visible.push((
                name.clone(),
                translated.clone(),
                parsed.statements.clone(),
                imports,
            ));
        }
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
    if version == 2 {
        for (module, source, statements, imports) in &visible {
            check_direct_import_visibility(module, source, statements, imports, &exports)?;
        }
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
    fn localized_source_imports_work_for_all_40_profiles() {
        for profile in crate::profiles::PROFILES {
            let import_word = crate::profiles::word(profile.code, "import").unwrap();
            let print_word = crate::profiles::word(profile.code, "print").unwrap();
            let source = format!(
                "# leading comment\n{import_word} math # direct dependency\n{print_word}(3)\n"
            );
            let (clean, imported) = source_imports(&source, profile.code, true).unwrap();
            assert!(imported.contains("math"), "{}", profile.code);
            assert_eq!(imported.len(), 1);
            assert!(clean.starts_with("# leading comment\n\n"));
            crate::parse_validated(&clean, profile.code).unwrap();
            let translated = crate::translate_source(&clean, profile.code, "ai").unwrap();
            assert!(crate::same_ir(&clean, profile.code, &translated, "ai").unwrap());
            assert_eq!(
                source_imports(&source, profile.code, false)
                    .unwrap_err()
                    .code,
                5003
            );
        }
    }

    #[test]
    fn source_imports_are_strict_and_do_not_interpret_literal_text() {
        for source in [
            "import math\nimport math\nprint(3)\n",
            "print(3)\nimport math\n",
            "import math more\nprint(3)\n",
            "import math,other\nprint(3)\n",
            "import Math\nprint(3)\n",
        ] {
            assert_eq!(source_imports(source, "ai", true).unwrap_err().code, 5003);
        }
        let triple = "print(\"\"\"example\nimport math\n\"\"\")\n";
        let (clean, imported) = source_imports(triple, "ai", true).unwrap();
        assert_eq!(clean, triple);
        assert!(imported.is_empty());
        let commented = "# import math\nprint(3)\n";
        let (clean, imported) = source_imports(commented, "ai", true).unwrap();
        assert_eq!(clean, commented);
        assert!(imported.is_empty());
        let (clean, imported) = source_imports("import math", "ai", true).unwrap();
        assert_eq!(clean, "");
        assert!(imported.contains("math"));
    }

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
