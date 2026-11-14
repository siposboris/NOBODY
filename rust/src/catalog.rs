//! Rust replacement for the historical Python language catalogue.
//! A listed reference variant is not necessarily an implemented syntax profile.
use std::collections::HashSet;

const REFERENCE: &str = include_str!("../../nobody/data/google_translate_249.txt");

const REVIEWED: &[(&str, &str)] = &[
    ("Slovak", "sk"),
    ("English", "en"),
    ("Czech", "cs"),
    ("German", "de"),
    ("Spanish", "es"),
    ("French", "fr"),
    ("Portuguese (Portugal)", "pt"),
    ("Russian", "ru"),
    ("Chinese (Simplified)", "zh"),
    ("Hindi", "hi"),
    ("Arabic", "ar"),
    ("Bengali", "bn"),
    ("Indonesian", "id"),
    ("Urdu", "ur"),
    ("Japanese", "ja"),
    ("Korean", "ko"),
    ("Turkish", "tr"),
    ("Vietnamese", "vi"),
    ("Tamil", "ta"),
    ("Italian", "it"),
    ("Thai", "th"),
    ("Polish", "pl"),
    ("Hebrew", "he"),
    ("Greek", "el"),
    ("Ukrainian", "uk"),
    ("Persian", "fa"),
    ("Swahili", "sw"),
    ("Dutch", "nl"),
    ("Bulgarian", "bg"),
    ("Romanian", "ro"),
    ("Swedish", "sv"),
    ("Finnish", "fi"),
    ("Armenian", "hy"),
    ("Georgian", "ka"),
    ("Nepali", "ne"),
    ("Marathi", "mr"),
    ("Gujarati", "gu"),
    ("Kannada", "kn"),
    ("Telugu", "te"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a> {
    pub name: &'a str,
    pub status: &'static str,
    pub syntax_code: Option<&'static str>,
}

pub fn languages() -> Result<Vec<Entry<'static>>, String> {
    let names: Vec<_> = REFERENCE.lines().collect();
    let unique: HashSet<_> = names.iter().copied().collect();
    if names.len() != 249 || unique.len() != 249 {
        return Err(format!(
            "Expected 249 unique language/script variants, found {} entries and {} unique names",
            names.len(),
            unique.len()
        ));
    }
    let reviewed_names: HashSet<_> = REVIEWED.iter().map(|(name, _)| *name).collect();
    let reviewed_codes: HashSet<_> = REVIEWED.iter().map(|(_, code)| *code).collect();
    if reviewed_names.len() != REVIEWED.len()
        || reviewed_codes.len() != REVIEWED.len()
        || !reviewed_names.is_subset(&unique)
    {
        return Err(
            "Reviewed language mappings must be unique and present in reference catalogue"
                .to_owned(),
        );
    }
    Ok(names
        .into_iter()
        .map(|name| {
            let code = REVIEWED
                .iter()
                .find(|(reviewed, _)| *reviewed == name)
                .map(|(_, code)| *code);
            Entry {
                name,
                status: if code.is_some() {
                    "experimental"
                } else {
                    "planned"
                },
                syntax_code: code,
            }
        })
        .collect())
}

pub fn summary(entries: &[Entry<'_>]) -> (usize, usize, usize) {
    (
        entries.len(),
        crate::profiles::PROFILES.len(),
        entries.iter().filter(|e| e.status == "planned").count(),
    )
}

fn json_string(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c < ' ' => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

pub fn json(entries: &[Entry<'_>]) -> String {
    let (total, active, planned) = summary(entries);
    let mut out = format!(
        "{{\n  \"summary\": {{\n    \"reference_variants\": {total},\n    \"active_experimental_profiles\": {active},\n    \"planned_reference_variants\": {planned}\n  }},\n  \"entries\": [\n"
    );
    for (index, entry) in entries.iter().enumerate() {
        if index != 0 {
            out.push_str(",\n");
        }
        let code = entry
            .syntax_code
            .map(json_string)
            .unwrap_or_else(|| "null".to_owned());
        out.push_str(&format!(
            "    {{\"name\": {}, \"status\": {}, \"syntax_code\": {code}}}",
            json_string(entry.name),
            json_string(entry.status)
        ));
    }
    out.push_str("\n  ]\n}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_matches_legacy_registration_without_activating_planned_profiles() {
        let entries = languages().unwrap();
        assert_eq!(summary(&entries), (249, 40, 210));
        assert_eq!(
            entries
                .iter()
                .find(|e| e.name == "Slovak")
                .unwrap()
                .syntax_code,
            Some("sk")
        );
        assert_eq!(
            entries
                .iter()
                .find(|e| e.name == "Chinese (Traditional)")
                .unwrap()
                .syntax_code,
            None
        );
        assert!(entries
            .iter()
            .filter(|e| e.status == "experimental")
            .all(|e| {
                crate::profiles::PROFILES
                    .iter()
                    .any(|p| Some(p.code) == e.syntax_code)
            }));
    }

    #[test]
    fn catalog_json_escapes_strings_and_reports_null_for_unimplemented_variants() {
        assert_eq!(json_string("a\"\\\n"), "\"a\\\"\\\\\\n\"");
        let output = json(&languages().unwrap());
        assert!(output.contains("\"reference_variants\": 249"));
        assert!(output.contains("\"planned_reference_variants\": 210"));
        assert!(output.contains(
            "\"name\": \"Slovak\", \"status\": \"experimental\", \"syntax_code\": \"sk\""
        ));
        assert!(output.contains(
            "\"name\": \"Chinese (Traditional)\", \"status\": \"planned\", \"syntax_code\": null"
        ));
    }
}
