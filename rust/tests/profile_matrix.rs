//! Cross-profile acceptance regression for the currently supported Rust grammar.
//! This checks structural IR only; it does not prove runtime equivalence or
//! native-speaker correctness of the experimental keyword dictionaries.
use nobody::profiles::PROFILES;
use nobody::{lower_source, translate_source};

const CANONICAL: &str = "fn twice(x) { return x * 2 }\nlet n: int = 2\nif true { print(twice(n)) } else { print(0) }\nwhile false { n = n + 1 }\nfor item in range(2) { print(item) }";

#[test]
fn all_40_profiles_parse_representative_control_flow_with_same_ir() {
    let baseline = lower_source(CANONICAL, "ai").expect("canonical grammar fixture must parse");
    for profile in PROFILES {
        let source = translate_source(CANONICAL, "ai", profile.code)
            .unwrap_or_else(|error| panic!("translation into {}: {error}", profile.code));
        let result = lower_source(&source, profile.code)
            .unwrap_or_else(|error| panic!("lowering {}: {error}", profile.code));
        assert_eq!(result, baseline, "profile {}", profile.code);
    }
}

#[test]
fn every_pair_of_40_profiles_preserves_canonical_ir() {
    let baseline = lower_source(CANONICAL, "ai").unwrap();
    for source_profile in PROFILES {
        let source = translate_source(CANONICAL, "ai", source_profile.code).unwrap();
        for target_profile in PROFILES {
            let translated = translate_source(&source, source_profile.code, target_profile.code)
                .unwrap_or_else(|error| {
                    panic!(
                        "{} -> {} translation failed: {error}",
                        source_profile.code, target_profile.code
                    )
                });
            let result = lower_source(&translated, target_profile.code).unwrap_or_else(|error| {
                panic!(
                    "{} -> {} lowering failed: {error}",
                    source_profile.code, target_profile.code
                )
            });
            assert_eq!(
                result, baseline,
                "IR mismatch in {} -> {}",
                source_profile.code, target_profile.code
            );
        }
    }
}
