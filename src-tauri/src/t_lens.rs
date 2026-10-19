/// Lens make inference utilities.
/// This is a best-effort fallback used when EXIF LensMake is missing.

enum RuleKind {
    StartsWith,
    Contains,
}

struct LensMakeRule {
    needle: &'static str,
    make: &'static str,
    kind: RuleKind,
}

fn normalize_model(input: &str) -> String {
    input
        .trim()
        .to_uppercase()
        .replace(['\t', '\n', '\r'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn matches_rule(model: &str, rule: &LensMakeRule) -> bool {
    match rule.kind {
        RuleKind::StartsWith => model.starts_with(rule.needle),
        RuleKind::Contains => model.contains(rule.needle),
    }
}

// Keep more specific rules first.
// Notes from vendor naming docs/examples:
// - Canon: EF/EF-S/EF-M and RF/RF-S prefixes.
// - Sony: model prefix SEL; lens naming prefixes FE / E / PZ.
// - Fujifilm: FUJINON XF/XC/GF.
// - Nikon: NIKKOR and NIKKOR Z / DX naming.
// - Tamron: Di / Di II / Di III / Di III-A terms commonly in names.
const LENS_MAKE_RULES: &[LensMakeRule] = &[
    // Canon
    LensMakeRule {
        needle: "CANON",
        make: "Canon",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "CN-E",
        make: "Canon",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "TS-E",
        make: "Canon",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "MP-E",
        make: "Canon",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "RF-S",
        make: "Canon",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "RF",
        make: "Canon",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "EF-S",
        make: "Canon",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "EF-M",
        make: "Canon",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "EF",
        make: "Canon",
        kind: RuleKind::StartsWith,
    },
    // Nikon
    LensMakeRule {
        needle: "NIKKOR",
        make: "Nikon",
        kind: RuleKind::Contains,
    },
    // Sony
    LensMakeRule {
        needle: "SONY",
        make: "Sony",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "SEL",
        make: "Sony",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "FE ",
        make: "Sony",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "E ",
        make: "Sony",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "E PZ ",
        make: "Sony",
        kind: RuleKind::StartsWith,
    },
    // Fujifilm
    LensMakeRule {
        needle: "FUJINON",
        make: "Fujifilm",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "XF",
        make: "Fujifilm",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "XC",
        make: "Fujifilm",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "GF",
        make: "Fujifilm",
        kind: RuleKind::StartsWith,
    },
    // Panasonic / Leica
    LensMakeRule {
        needle: "LUMIX",
        make: "Panasonic",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "LEICA DG",
        make: "Panasonic",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "LEICA D",
        make: "Panasonic",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "LEICA",
        make: "Leica",
        kind: RuleKind::StartsWith,
    },
    // Tamron
    LensMakeRule {
        needle: "TAMRON",
        make: "Tamron",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: " DI III-A",
        make: "Tamron",
        kind: RuleKind::Contains,
    },
    LensMakeRule {
        needle: " DI III",
        make: "Tamron",
        kind: RuleKind::Contains,
    },
    LensMakeRule {
        needle: " DI II",
        make: "Tamron",
        kind: RuleKind::Contains,
    },
    LensMakeRule {
        needle: " DI ",
        make: "Tamron",
        kind: RuleKind::Contains,
    },
    // Sigma
    LensMakeRule {
        needle: "SIGMA",
        make: "Sigma",
        kind: RuleKind::StartsWith,
    },
    // Others
    LensMakeRule {
        needle: "TOKINA",
        make: "Tokina",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "AT-X",
        make: "Tokina",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "SAMYANG",
        make: "Samyang",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "ROKINON",
        make: "Samyang",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "VOIGTLANDER",
        make: "Voigtlander",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "TTARTISAN",
        make: "TTArtisan",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "7ARTISANS",
        make: "7Artisans",
        kind: RuleKind::StartsWith,
    },
    LensMakeRule {
        needle: "VILTROX",
        make: "Viltrox",
        kind: RuleKind::StartsWith,
    },
];

/// Infer lens make from lens model naming patterns.
pub fn infer_lens_make(lens_model: &str) -> Option<&'static str> {
    let normalized = normalize_model(lens_model);
    if normalized.is_empty() {
        return None;
    }

    for rule in LENS_MAKE_RULES {
        if matches_rule(&normalized, rule) {
            return Some(rule.make);
        }
    }
    None
}
