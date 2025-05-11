pub mod encoder;
pub mod java_script_encoder;

pub mod encoders {
    use crate::encoder::{CharRule, CharRuleType, EncodeType, CustomEncoder, simple_hex_encode};

    pub fn java_script_encoder() -> CustomEncoder {
        let rules: Vec<CharRule> = vec![
            CharRule::Range {
                start: '\u{00000}',
                end: '\u{0001F}',
                exclude: Some(vec![
                    '\u{0000}', '\u{0008}', '\u{0009}', '\u{000A}', '\u{000C}', '\u{000D}',
                ]),
                rule_type: CharRuleType::Allow,
            },
            CharRule::Single {
                c: '\u{0008}',
                rule_type: CharRuleType::Replace("\\b"),
            },
            CharRule::Single {
                c: '\u{0009}',
                rule_type: CharRuleType::Replace("\\t"),
            },
            CharRule::Single {
                c: '\u{000A}',
                rule_type: CharRuleType::Replace("\\n"),
            },
            CharRule::Single {
                c: '\u{000C}',
                rule_type: CharRuleType::Replace("\\f"),
            },
            CharRule::Single {
                c: '\u{000D}',
                rule_type: CharRuleType::Replace("\\r"),
            },
            CharRule::Single {
                c: '\u{0000}',
                rule_type: CharRuleType::Encode(EncodeType::Hex(simple_hex_encode)),
            },
            CharRule::Range {
                start: ' ',
                end: '~',
                exclude: Some(vec!['"', '\\', '\'', '-', '/', '&', '`']),
                rule_type: CharRuleType::Allow,
            },
            CharRule::Single {
                c: '/',
                rule_type: CharRuleType::Escape {
                    simple_escape: true,
                    min_len: true,
                },
            },
            CharRule::Single {
                c: '-',
                rule_type: CharRuleType::Escape {
                    simple_escape: true,
                    min_len: true,
                },
            },
            CharRule::Single {
                c: '"',
                rule_type: CharRuleType::Escape {
                    simple_escape: true,
                    min_len: true,
                },
            },
            CharRule::Single {
                c: '\'',
                rule_type: CharRuleType::Escape {
                    simple_escape: true,
                    min_len: true,
                },
            },
            CharRule::Single {
                c: '\u{2028}',
                rule_type: CharRuleType::Encode(EncodeType::Hex(simple_hex_encode)),
            },
            CharRule::Single {
                c: '\u{2029}',
                rule_type: CharRuleType::Encode(EncodeType::Hex(simple_hex_encode)),
            },
            CharRule::Single {
                c: '&',
                rule_type: CharRuleType::Escape {
                    simple_escape: true,
                    min_len: true,
                },
            },
            // escape all non-ascii chars
            CharRule::Range {
                start: '\u{00FF}',
                end: '\u{FFFFF}',
                exclude: None,
                rule_type: CharRuleType::Escape {
                    simple_escape: false,
                    min_len: true,
                },
            },
        ];
        CustomEncoder::new(rules, EncodeType::Hex(simple_hex_encode), '\\', '\u{FFFD}')
    }
}


