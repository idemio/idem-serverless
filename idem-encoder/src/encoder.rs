
// TODO - This implementation is much more flexible than the other encoders. But is much slower. This will be a work in progress in the meantime.

const HEX_SHIFT: u32 = 4;
const HEX_MASK: u32 = 0x0F;
const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

pub enum EncodeType {
    Hex(fn(char) -> String),
}

pub enum CharRuleType {
    Allow,
    Deny,
    Encode(EncodeType),
    Escape { min_len: bool, simple_escape: bool },
    Replace(&'static str),
}
pub enum CharRule {
    Range {
        start: char,
        end: char,
        exclude: Option<Vec<char>>,
        rule_type: CharRuleType,
    },
    Single {
        c: char,
        rule_type: CharRuleType,
    },
}

pub struct CustomEncoder {
    pub(crate) rules: Vec<CharRule>,
    pub(crate) encode_type: EncodeType,
    pub(crate) escape_char: char,
    pub(crate) invalid_char: char,
    pub(crate) output_buffer_max_len: usize,
}

impl CustomEncoder {
    pub fn new(
        rules: Vec<CharRule>,
        encode_type: EncodeType,
        escape_char: char,
        invalid_char: char,
    ) -> Self {
        let mut max_len = 1usize;
        for rule in rules.iter() {
            match rule {
                CharRule::Range { rule_type, .. } => {
                    if let CharRuleType::Replace(s) = rule_type {
                        max_len = max_len.max(s.len());
                    }
                }
                CharRule::Single { rule_type, .. } => {
                    if let CharRuleType::Replace(s) = rule_type {
                        max_len = max_len.max(s.len());
                    }
                }
            }
        }
        log::debug!("Max replace length found was '{}' (1 is default)", max_len);
        Self {
            rules,
            encode_type,
            escape_char,
            invalid_char,
            output_buffer_max_len: max_len,
        }
    }

    pub fn encode(&self, input: &str) -> String {
        let mut output = String::with_capacity(input.len() * self.output_buffer_max_len);

        for ch in input.chars() {
            let mut rule_applied = false;
            'inner: for rule in &self.rules {
                match rule {
                    CharRule::Range {
                        start,
                        end,
                        exclude,
                        rule_type,
                    } => {
                        if exclude.as_ref().is_some_and(|exl| exl.contains(&ch)) {
                            continue 'inner;
                        } else if (ch as u32) >= (*start as u32) && (ch as u32) <= (*end as u32) {
                            self.handle_char_rule(rule_type, ch, &mut output);
                            rule_applied = true;
                            break 'inner;
                        }
                    }
                    CharRule::Single { c, rule_type } => {
                        if ch == *c {
                            self.handle_char_rule(rule_type, ch, &mut output);
                            rule_applied = true;
                            break 'inner;
                        }
                    }
                }
            }

            if !rule_applied {
                log::warn!("No rules found for character: '{}'", ch);
                output.push(ch);
            }
        }
        output.shrink_to_fit();
        output
    }

    fn handle_char_rule(&self, rule: &CharRuleType, ch: char, output: &mut String) {
        match rule {
            CharRuleType::Allow => {
                output.push(ch);
            }
            CharRuleType::Deny => {
                output.push(self.invalid_char);
            }
            CharRuleType::Encode(encode_type) => match encode_type {
                EncodeType::Hex(function) => {
                    output.push_str(&function(ch));
                }
            },
            CharRuleType::Escape {
                simple_escape,
                min_len,
            } => {
                if *simple_escape {
                    output.push(self.escape_char);
                    output.push(ch);
                    return;
                } else if *min_len && ch as u32 <= 0xFF {
                    output.push('\\');
                    output.push('x');
                    output.push(HEX[(ch as u32 >> HEX_SHIFT) as usize]);
                    output.push(HEX[(ch as u32 & HEX_MASK) as usize]);
                    return;
                }

                output.push(self.escape_char);
                output.push('u');
                output.push(HEX[(ch as u32 >> (3 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
                output.push(HEX[(ch as u32 >> (2 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
                output.push(HEX[(ch as u32 >> (1 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
                output.push(HEX[(ch as u32 & HEX_MASK) as usize]);
            }
            CharRuleType::Replace(replace) => {
                output.push_str(&replace);
            }
        }
    }
}

pub(crate) fn simple_hex_encode(input: char) -> String {
    let mut result = String::with_capacity(6);
    if input as u32 <= 0xFF {
        result.push('\\');
        result.push('x');
        result.push(HEX[(input as u32 >> HEX_SHIFT) as usize]);
        result.push(HEX[(input as u32 & HEX_MASK) as usize]);
    } else {
        result.push('\\');
        result.push('u');
        result.push(HEX[(input as u32 >> (3 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
        result.push(HEX[(input as u32 >> (2 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
        result.push(HEX[(input as u32 >> (1 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
        result.push(HEX[(input as u32 & HEX_MASK) as usize]);
    }
    result.shrink_to_fit();
    result
}

#[cfg(test)]
mod test {
    use crate::encoder::{CharRule, CharRuleType, EncodeType, CustomEncoder, simple_hex_encode};

    #[test]
    fn simple_ascii_only_encode() {
        let rules: Vec<CharRule> = vec![
            CharRule::Range {
                start: '\u{00000}',
                end: '\u{0007F}',
                exclude: None,
                rule_type: CharRuleType::Allow,
            },
            CharRule::Range {
                start: '\u{00080}',
                end: '\u{FFFFF}',
                exclude: None,
                rule_type: CharRuleType::Deny,
            },
        ];

        let encoder = CustomEncoder::new(rules, EncodeType::Hex(simple_hex_encode), '\\', '�');
        let output = encoder.encode(
            r#"
        This is a test to see if any characters are changed.
        more tests  .😀.😀.😀.😀.
        "#,
        );
        let expected = r#"
        This is a test to see if any characters are changed.
        more tests  .�.�.�.�.
        "#;
        println!("output: {:?}", output);
        assert_eq!(output, expected);
    }

    fn generic_tests(encoder: &CustomEncoder) {
        let backspace_test = encoder.encode(&'\u{0008}'.to_string());
        assert_eq!("\\b", backspace_test);

        let tab_test = encoder.encode(&'\t'.to_string());
        assert_eq!("\\t", tab_test);

        let newline_test = encoder.encode(&'\n'.to_string());
        assert_eq!("\\n", newline_test);

        let carriage_return_test = encoder.encode(&'\r'.to_string());
        assert_eq!("\\r", carriage_return_test);

        let nul_test = encoder.encode(&'\u{0000}'.to_string());
        assert_eq!("\\x00", nul_test);

        let line_separator_test = encoder.encode(&'\u{2028}'.to_string());
        let line_separator_assertion = "\\u2028".to_string();
        assert_eq!(line_separator_test, line_separator_test);

        let paragraph_separator_test = encoder.encode(&'\u{2029}'.to_string());
        let paragraph_separator_assertion = "\\u2029".to_string();
        assert_eq!(paragraph_separator_assertion, paragraph_separator_test);

        let simple_lower_case_test = encoder.encode(&"abcd".to_string());
        assert_eq!("abcd", simple_lower_case_test);

        let simple_upper_case_test = encoder.encode(&"ABCD".to_string());
        assert_eq!("ABCD", simple_upper_case_test);
    }

    fn ascii_only_tests(encoder: &CustomEncoder) {
        let simple_unicode_test = encoder.encode(&'\u{1234}'.to_string());
        assert_eq!("\\u1234", simple_unicode_test);

        let high_ascii_test = encoder.encode(&'\u{00ff}'.to_string());
        assert_eq!("\\xff", high_ascii_test);
    }

    #[test]
    fn javascript_block_encode_test() {
        env_logger::init();
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
        let encoder = CustomEncoder::new(rules, EncodeType::Hex(simple_hex_encode), '\\', '�');
        generic_tests(&encoder);
        ascii_only_tests(&encoder);
        let double_quote_test = encoder.encode(&'"'.to_string());
        assert_eq!("\\\"", double_quote_test);

        let single_quote_test = encoder.encode(&'\''.to_string());
        assert_eq!("\\\'", single_quote_test);
    }
}
