const HEX_SHIFT: u32 = 4;
const HEX_MASK: u32 = 0x0F;
const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
pub(crate) fn simple_hex_encode(input: char) -> String {
    let mut result = String::with_capacity(6);
    if input as u32 <= 0xFF {
        result.push('\\');
        result.push('x');
        result.push(HEX[(input as u32 >> HEX_SHIFT) as usize]);
        result.push(HEX[(input as u32 & HEX_MASK) as usize]);
        result
    } else {
        result.push('\\');
        result.push('u');
        result.push(HEX[(input as u32 >> (3 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
        result.push(HEX[(input as u32 >> (2 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
        result.push(HEX[(input as u32 >> (1 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
        result.push(HEX[(input as u32 & HEX_MASK) as usize]);
        result
    }
}

pub(crate) enum EncodeType {
    Hex(fn(char) -> String),
}

pub(crate) enum CharRuleType {
    Allow,
    Deny,
    Encode(EncodeType),
    Escape,
    Replace(String),
}
pub(crate) enum CharRule {
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

pub(crate) struct EncoderV2 {
    pub(crate) rules: Vec<CharRule>,
    pub(crate) encode_type: EncodeType,
    pub(crate) escape_char: char,
    pub(crate) invalid_char: char,
    pub(crate) output_buffer_max_len: usize,
}

impl EncoderV2 {
    pub fn new(rules: Vec<CharRule>, encode_type: EncodeType, escape_char: char, invalid_char: char) -> Self {
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
                        if ch.eq(c) {
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
            },
            CharRuleType::Deny => {
                output.push(self.invalid_char);
            },
            CharRuleType::Encode(encode_type) => match encode_type {
                EncodeType::Hex(function) => {
                    output.push_str(&function(ch));
                },
            },
            CharRuleType::Escape => {
                output.push(self.escape_char);
                output.push(ch);
            }
            CharRuleType::Replace(replace) => {
                output.push_str(&replace);
            },
        }
    }
}




#[cfg(test)]
mod test {
    use crate::encoder::{CharRule, CharRuleType, EncoderV2, EncodeType, simple_hex_encode};

    #[test]
    fn simple_ascii_only_encode() {
        let rules: Vec<CharRule> = vec![
            CharRule::Range {
                start: '\u{00000}',
                end: '\u{0007F}',
                exclude: None,
                rule_type: CharRuleType::Allow
            },
            CharRule::Range {
                start: '\u{00080}',
                end: '\u{FFFFF}',
                exclude: None,
                rule_type: CharRuleType::Deny
            }
        ];

        let encoder = EncoderV2::new(rules, EncodeType::Hex(simple_hex_encode), '\\', '�');
        let output = encoder.encode(r#"
        This is a test to see if any characters are changed.
        more tests  .😀.😀.😀.😀.
        "#);
        let expected = r#"
        This is a test to see if any characters are changed.
        more tests  .�.�.�.�.
        "#;
        println!("output: {:?}", output);
        assert_eq!(output, expected);
    }

    #[test]
    fn javascript_block_encode_test() {
        let rules: Vec<CharRule> = vec![

            CharRule::Range {
                start: '\u{00000}',
                end: '\u{0001F}',
                exclude: Some(vec![
                    '\u{0008}', '\u{0009}', '\u{000A}', '\u{000C}', '\u{000D}'
                ]),
                rule_type: CharRuleType::Allow,
            },

            CharRule::Single {
                c: '\u{0008}',
                rule_type: CharRuleType::Escape,
            },

            CharRule::Single {
                c: '\u{0009}',
                rule_type: CharRuleType::Escape,
            },

            CharRule::Single {
                c: '\u{000A}',
                rule_type: CharRuleType::Escape,
            },

            CharRule::Single {
                c: '\u{000C}',
                rule_type: CharRuleType::Escape,
            },

            CharRule::Single {
                c: '\u{000D}',
                rule_type: CharRuleType::Escape,
            },

            CharRule::Range {
                start: ' ',
                end: '~',
                exclude: Some(vec![
                    '"', '\\', '\'', '-', '/', '&', '`'
                ]),
                rule_type: CharRuleType::Allow
            },

            CharRule::Single {
                c: '/',
                rule_type: CharRuleType::Escape,
            },

            CharRule::Single {
                c: '-',
                rule_type: CharRuleType::Escape,
            },

            CharRule::Single {
                c: '"',
                rule_type: CharRuleType::Escape,
            },

            CharRule::Single {
                c: '\'',
                rule_type: CharRuleType::Escape,
            },

            CharRule::Single {
                c: '&',
                rule_type: CharRuleType::Escape,
            }
        ];

        let encoder = EncoderV2::new(rules, EncodeType::Hex(simple_hex_encode), '\\', '�');
        let input_example = "<script>print(\"😀\")</script>".to_string();
        let output = encoder.encode(&input_example);
        println!("out: {}", output)
    }
}

