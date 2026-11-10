//! Project v3's explicitly-qualified, name-isolated module linking.
use crate::syntax::{Expr, Program, Statement};
use crate::{scan, Diagnostic, TokenKind};
use std::collections::BTreeSet;
use std::fmt::Write;

const PRIVATE_PREFIX: &str = "nobody_ns_";
const MAX_SYMBOL_BYTES: usize = 128;

fn invalid(detail: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code: 5003,
        line: 0,
        column: 0,
        detail: detail.into(),
        repair: "Use a declared, directly imported module and its exported top-level symbol; qualified calls use module.symbol(...).",
    }
}

/// Stable collision-free ASCII name for a module and UTF-8 public symbol.
/// Hex escaping preserves non-Latin user identifiers and cannot collide
/// with other module/symbol pairs.
pub(crate) fn private_name(module: &str, symbol: &str) -> String {
    let mut result = format!("{PRIVATE_PREFIX}{module}_x");
    for byte in symbol.bytes() {
        write!(&mut result, "{byte:02x}").expect("string write cannot fail");
    }
    result
}

/// Compile-time desugaring of member syntax into an unambiguous private
/// identifier, before the ordinary single-file AST parser sees the code.
/// Strings/comments are untouched, and user identifiers cannot forge the
/// reserved internal name prefix. The source-level qualification check is
/// deliberately restricted to direct module imports, not arbitrary objects.
pub(crate) fn qualify_references(
    source: &str,
    module: &str,
    imports: &BTreeSet<String>,
) -> Result<(String, Vec<(String, String)>), Diagnostic> {
    let tokens = scan(source)?;
    for token in &tokens {
        if token.kind == TokenKind::Name && token.value.starts_with(PRIVATE_PREFIX) {
            return Err(invalid(format!(
                "Module {module:?} uses reserved compiler-internal identifier {:?}",
                token.value
            )));
        }
    }
    let mut result = String::with_capacity(source.len());
    let mut at = 0usize;
    let mut refs = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if index + 2 < tokens.len()
            && tokens[index].kind == TokenKind::Name
            && tokens[index + 1].kind == TokenKind::Symbol
            && tokens[index + 1].value == "."
            && tokens[index + 2].kind == TokenKind::Name
        {
            let target_module = &tokens[index].value;
            let symbol = &tokens[index + 2].value;
            if !imports.contains(target_module) {
                return Err(invalid(format!(
                    "Module {module:?} accesses {target_module}.{symbol} without directly importing {target_module:?}"
                )));
            }
            if symbol.len() > MAX_SYMBOL_BYTES {
                return Err(invalid("Qualified module export identifier exceeds 128 UTF-8 bytes"));
            }
            let generated = private_name(target_module, symbol);
            result.push_str(&source[at..tokens[index].start]);
            result.push_str(&generated);
            at = tokens[index + 2].end;
            refs.push((target_module.clone(), symbol.clone()));
            index += 3;
            continue;
        }
        index += 1;
    }
    result.push_str(&source[at..]);
    Ok((result, refs))
}

fn rename_expr(expr: &mut Expr, module: &str, exports: &BTreeSet<String>, locals: &BTreeSet<String>) {
    match expr {
        Expr::Name(name) => {
            if !locals.contains(name) && exports.contains(name) {
                *name = private_name(module, name);
            }
        }
        Expr::Call { callee, args } => {
            rename_expr(callee, module, exports, locals);
            for arg in args {
                rename_expr(arg, module, exports, locals);
            }
        }
        Expr::Unary { value, .. } => rename_expr(value, module, exports, locals),
        Expr::Binary { lhs, rhs, .. } => {
            rename_expr(lhs, module, exports, locals);
            rename_expr(rhs, module, exports, locals);
        }
        _ => {}
    }
}

fn rename_block(
    statements: &mut [Statement],
    module: &str,
    exports: &BTreeSet<String>,
    locals: &mut BTreeSet<String>,
    top_level: bool,
) {
    for statement in statements {
        match statement {
            Statement::Let { name, expr, .. } => {
                rename_expr(expr, module, exports, locals);
                if top_level && exports.contains(name) {
                    *name = private_name(module, name);
                } else {
                    locals.insert(name.clone());
                }
            }
            Statement::Assign { name, expr } => {
                rename_expr(expr, module, exports, locals);
                if !locals.contains(name) && exports.contains(name) {
                    *name = private_name(module, name);
                }
            }
            Statement::Function { name, parameters, body, .. } => {
                let mut inside = locals.clone();
                inside.extend(parameters.iter().cloned());
                if top_level && exports.contains(name) {
                    *name = private_name(module, name);
                } else {
                    inside.insert(name.clone());
                    locals.insert(name.clone());
                }
                rename_block(body, module, exports, &mut inside, false);
            }
            Statement::If { branches, otherwise } => {
                for (condition, body) in branches {
                    rename_expr(condition, module, exports, locals);
                    rename_block(body, module, exports, &mut locals.clone(), false);
                }
                if let Some(body) = otherwise {
                    rename_block(body, module, exports, &mut locals.clone(), false);
                }
            }
            Statement::While { condition, body } => {
                rename_expr(condition, module, exports, locals);
                rename_block(body, module, exports, &mut locals.clone(), false);
            }
            Statement::For { variable, iterable, body } => {
                rename_expr(iterable, module, exports, locals);
                let mut scoped = locals.clone();
                scoped.insert(variable.clone());
                rename_block(body, module, exports, &mut scoped, false);
            }
            Statement::Return(expr) | Statement::Expression(expr) => {
                rename_expr(expr, module, exports, locals);
            }
            Statement::Break | Statement::Continue => {}
        }
    }
}

/// Returns the ordinary canonical ai source accepted by the existing
/// Rust AST evaluator, portable bytecode compiler and native ELF backend.
pub(crate) fn assemble_namespaced_module(
    program: &mut Program,
    module: &str,
    exports: &BTreeSet<String>,
) -> Result<String, Diagnostic> {
    rename_block(&mut program.statements, module, exports, &mut BTreeSet::new(), true);
    crate::emit_program(program, "ai")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_collision_free_and_ascii_even_for_unicode_exports() {
        assert_ne!(private_name("math", "step"), private_name("extra", "step"));
        assert_ne!(private_name("math", "a"), private_name("math", "b"));
        let encoded = private_name("math", "živý");
        assert!(encoded.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'));
        assert!(encoded.starts_with("nobody_ns_math_x"));
    }

    #[test]
    fn reference_rewriting_is_import_scoped_and_does_not_change_literals() {
        let imports = BTreeSet::from(["math".to_owned()]);
        let source = "print(\"math.step\") # math.step\nprint(math.step(4))\n";
        let (rewritten, refs) = qualify_references(source, "main", &imports).unwrap();
        assert_eq!(refs, vec![("math".to_owned(), "step".to_owned())]);
        assert!(rewritten.contains("\"math.step\""));
        assert!(rewritten.contains("# math.step"));
        assert!(rewritten.contains(&private_name("math", "step")));
        assert_eq!(qualify_references("print(unknown.step(1))", "main", &imports)
            .unwrap_err().code, 5003);
        assert_eq!(qualify_references("print(nobody_ns_fake(1))", "main", &imports)
            .unwrap_err().code, 5003);
    }

    #[test]
    fn top_level_names_are_mangled_but_shadowing_parameters_are_preserved() {
        let program = "fn step(n: int): int { return n * 2 }\nfn shadow(step: int): int { return step + 1 }\nprint(step(3), shadow(6))\n";
        let mut parsed = crate::parse_validated(program, "ai").unwrap();
        let exports = BTreeSet::from(["step".to_owned(), "shadow".to_owned()]);
        let compiled = assemble_namespaced_module(&mut parsed, "math", &exports).unwrap();
        assert!(compiled.contains(&private_name("math", "step")));
        assert!(compiled.contains(&private_name("math", "shadow")));
        assert!(compiled.contains("return (step + 1)"));
        assert_eq!(crate::evaluate_source(&compiled, "ai").unwrap(), "6 7\n");
    }
}
