//! Rust-only README roadmap checklist counter. This number is not compiler completion.
use std::{env, fs, path::Path};

const BEGIN: &str = "<!-- roadmap-progress:start -->";
const END: &str = "<!-- roadmap-progress:end -->";
const COMMAND: &str =
    "cargo run --locked --manifest-path rust/Cargo.toml --bin nobody-roadmap -- --write";

fn render(roadmap: &str) -> Result<String, String> {
    let (mut complete, mut total) = (0usize, 0usize);
    for line in roadmap.lines() {
        if line.starts_with("- [x] ") {
            complete += 1;
            total += 1;
        } else if line.starts_with("- [ ] ") {
            total += 1;
        }
    }
    if total == 0 {
        return Err("No roadmap checkboxes found; refusing to report 100%".into());
    }
    let percent = (100 * complete + total / 2) / total;
    let filled = (20 * complete + total / 2) / total;
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(20 - filled));
    Ok(format!(
        "{BEGIN}\n\
### Roadmap checklist progress — {percent}%\n\n\
**{complete} / {total} roadmap checklist items marked complete** ({} still open)\n\n\
`{bar}` **{percent}%**\n\n\
This measures **completed checklist entries**, not the percentage of \
a working compiler, feature parity, quality, or production readiness. \
Some checked items are historical prototypes or partial milestones. \
Nobody is experimental; its Rust runtime, native compiler and \
self-hosting are not complete.\n\n\
See the [full roadmap](docs/ROADMAP.md) and \
[feature status](docs/FEATURES.md). \
Regenerate with `{COMMAND}`.\n\
{END}",
        total - complete
    ))
}

fn replace_block(readme: &str, replacement: &str) -> Result<String, String> {
    if readme.matches(BEGIN).count() != 1 || readme.matches(END).count() != 1 {
        return Err("README must contain exactly one progress marker pair".into());
    }
    let start = readme.find(BEGIN).expect("unique begin marker");
    let end = readme.find(END).expect("unique end marker");
    if end < start {
        return Err("README progress markers are reversed".into());
    }
    Ok(format!(
        "{}{}{}",
        &readme[..start],
        replacement,
        &readme[end + END.len()..]
    ))
}

fn run(mode: &str, root: &Path) -> Result<(), String> {
    let roadmap = fs::read_to_string(root.join("docs/ROADMAP.md")).map_err(|e| e.to_string())?;
    let path = root.join("README.md");
    let current = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let updated = replace_block(&current, &render(&roadmap)?)?;
    match mode {
        "--check" => {
            if current != updated {
                return Err(format!("Roadmap progress is stale. Run: {COMMAND}"));
            }
            println!("Roadmap progress is up to date.");
        }
        "--write" => {
            if current != updated {
                fs::write(&path, updated).map_err(|e| e.to_string())?;
            }
            println!("Roadmap progress updated.");
        }
        _ => return Err("Usage: nobody-roadmap [--check | --write]".into()),
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mode = match args.as_slice() {
        [] => "--check",
        [arg] if arg == "--check" || arg == "--write" => arg.as_str(),
        _ => {
            eprintln!("N9001: Usage: nobody-roadmap [--check | --write]");
            std::process::exit(2);
        }
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repository root");
    if let Err(message) = run(mode, root) {
        eprintln!("N9001: {message}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_and_rounds_like_legacy_python() {
        let rendered = render("- [x] done\n- [ ] todo\n").unwrap();
        assert!(rendered.contains("progress — 50%"));
        assert!(rendered.contains("**1 / 2 roadmap checklist items marked complete**"));
        assert!(rendered.contains("██████████░░░░░░░░░░"));
        assert!(rendered.contains(COMMAND));
    }
    #[test]
    fn nested_checkboxes_are_excluded_and_missing_checkboxes_fail() {
        let result = render("- [x] yes\n  - [x] nested\n- [ ] no\n- [x] yes\n").unwrap();
        assert!(result.contains("**2 / 3 roadmap checklist items marked complete**"));
        assert!(render("none\n").is_err());
    }
    #[test]
    fn markers_are_ordered_unique_and_idempotent() {
        let original = format!("INTRO\n{BEGIN}\nOLD\n{END}\nEND\n");
        let block = render("- [x] done\n").unwrap();
        let next = replace_block(&original, &block).unwrap();
        assert!(next.starts_with("INTRO\n"));
        assert!(next.ends_with("\nEND\n"));
        assert_eq!(replace_block(&next, &block).unwrap(), next);
        assert!(replace_block(&format!("{END}\n{BEGIN}"), &block).is_err());
        assert!(replace_block(&format!("{BEGIN}\n{BEGIN}\n{END}"), &block).is_err());
        assert!(replace_block("absent", &block).is_err());
    }
}
