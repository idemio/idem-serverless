use crate::encoder::{CompiledEncoderRules, Encoder, Rule, RuleType, ValidAsciiRange};

pub enum JavaScriptEncoderMode {
    Source,
    Block,
    Html,
    Attribute,
}

pub struct JavaScriptEncoderSettings {
    pub mode: JavaScriptEncoderMode,
}

pub struct JavaScriptEncoder {
    pub(crate) rules: CompiledEncoderRules,
    pub(crate) escape_char: char,
    pub(crate) invalid_char: char,
    pub(crate) ascii_properties: ValidAsciiRange,
}

impl Encoder<JavaScriptEncoderSettings> for JavaScriptEncoder {
    fn create(
        escape_char: char,
        valid_ascii_range: ValidAsciiRange,
        config: JavaScriptEncoderSettings,
    ) -> Self {
        match config.mode {
            JavaScriptEncoderMode::Source => {
                let rules = [
                    Rule::Uni {
                        ch: '\u{0008}',
                        rt: RuleType::Sub("\\b"),
                    },
                    Rule::Uni {
                        ch: '\u{0009}',
                        rt: RuleType::Sub("\\t"),
                    },
                    Rule::Uni {
                        ch: '\u{000A}',
                        rt: RuleType::Sub("\\n"),
                    },
                    Rule::Uni {
                        ch: '\u{000C}',
                        rt: RuleType::Sub("\\f"),
                    },
                    Rule::Uni {
                        ch: '\u{000D}',
                        rt: RuleType::Sub("\\r"),
                    },
                    Rule::Uni {
                        ch: '\u{0000}',
                        rt: RuleType::Esc {
                            min: true,
                            simple: false,
                        },
                    },
                    Rule::Uni {
                        ch: '"',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                    Rule::Uni {
                        ch: '\'',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                    Rule::Uni {
                        ch: '\\',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                ];
                let compiled_rules = Self::compile_encoder_rules(rules);
                Self {
                    rules: compiled_rules,
                    escape_char,
                    invalid_char: ' ',
                    ascii_properties: valid_ascii_range,
                }
            },
            JavaScriptEncoderMode::Block => {
                let rules = [
                    Rule::Uni {
                        ch: '\u{0008}',
                        rt: RuleType::Sub("\\b"),
                    },
                    Rule::Uni {
                        ch: '\u{0009}',
                        rt: RuleType::Sub("\\t"),
                    },
                    Rule::Uni {
                        ch: '\u{000A}',
                        rt: RuleType::Sub("\\n"),
                    },
                    Rule::Uni {
                        ch: '\u{000C}',
                        rt: RuleType::Sub("\\f"),
                    },
                    Rule::Uni {
                        ch: '\u{000D}',
                        rt: RuleType::Sub("\\r"),
                    },
                    Rule::Uni {
                        ch: '\u{0000}',
                        rt: RuleType::Esc {
                            min: true,
                            simple: false,
                        },
                    },
                    Rule::Uni {
                        ch: '/',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                    Rule::Uni {
                        ch: '-',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                    Rule::Uni {
                        ch: '"',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                    Rule::Uni {
                        ch: '\'',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                    Rule::Uni {
                        ch: '&',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                    Rule::Uni {
                        ch: '\\',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                ];
                let compiled_rules = Self::compile_encoder_rules(rules);
                Self {
                    rules: compiled_rules,
                    escape_char,
                    invalid_char: ' ',
                    ascii_properties: valid_ascii_range,
                }
            }
            JavaScriptEncoderMode::Html => {
                let rules = [
                    Rule::Uni {
                        ch: '\u{0008}',
                        rt: RuleType::Sub("\\b"),
                    },
                    Rule::Uni {
                        ch: '\u{0009}',
                        rt: RuleType::Sub("\\t"),
                    },
                    Rule::Uni {
                        ch: '\u{000A}',
                        rt: RuleType::Sub("\\n"),
                    },
                    Rule::Uni {
                        ch: '\u{000C}',
                        rt: RuleType::Sub("\\f"),
                    },
                    Rule::Uni {
                        ch: '\u{000D}',
                        rt: RuleType::Sub("\\r"),
                    },
                    Rule::Uni {
                        ch: '\u{0000}',
                        rt: RuleType::Esc {
                            min: true,
                            simple: false,
                        },
                    },
                    Rule::Uni {
                        ch: '/',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                    Rule::Uni {
                        ch: '-',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                    Rule::Uni {
                        ch: '"',
                        rt: RuleType::Esc {
                            simple: false,
                            min: false,
                        },
                    },
                    Rule::Uni {
                        ch: '\'',
                        rt: RuleType::Esc {
                            simple: false,
                            min: false,
                        },
                    },
                    Rule::Uni {
                        ch: '&',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                    Rule::Uni {
                        ch: '\\',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                ];
                let compiled_rules = Self::compile_encoder_rules(rules);
                Self {
                    rules: compiled_rules,
                    escape_char,
                    invalid_char: ' ',
                    ascii_properties: valid_ascii_range,
                }
            },
            JavaScriptEncoderMode::Attribute => {
                let rules = [
                    Rule::Uni {
                        ch: '\u{0008}',
                        rt: RuleType::Sub("\\b"),
                    },
                    Rule::Uni {
                        ch: '\u{0009}',
                        rt: RuleType::Sub("\\t"),
                    },
                    Rule::Uni {
                        ch: '\u{000A}',
                        rt: RuleType::Sub("\\n"),
                    },
                    Rule::Uni {
                        ch: '\u{000C}',
                        rt: RuleType::Sub("\\f"),
                    },
                    Rule::Uni {
                        ch: '\u{000D}',
                        rt: RuleType::Sub("\\r"),
                    },
                    Rule::Uni {
                        ch: '\u{0000}',
                        rt: RuleType::Esc {
                            min: true,
                            simple: false,
                        },
                    },
                    Rule::Uni {
                        ch: '"',
                        rt: RuleType::Esc {
                            simple: false,
                            min: false,
                        },
                    },
                    Rule::Uni {
                        ch: '\'',
                        rt: RuleType::Esc {
                            simple: false,
                            min: false,
                        },
                    },
                    Rule::Uni {
                        ch: '&',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                    Rule::Uni {
                        ch: '\\',
                        rt: RuleType::Esc {
                            simple: true,
                            min: true,
                        },
                    },
                ];
                let compiled_rules = Self::compile_encoder_rules(rules);
                Self {
                    rules: compiled_rules,
                    escape_char,
                    invalid_char: ' ',
                    ascii_properties: valid_ascii_range,
                }
            }
        }
    }

    fn compiled_rules(&self) -> &CompiledEncoderRules {
        &self.rules
    }

    fn escape_char(&self) -> char {
        self.escape_char
    }

    fn ascii_properties(&self) -> &ValidAsciiRange {
        &self.ascii_properties
    }
}

#[cfg(test)]
mod test {
    use crate::encoder::*;
    use crate::java_script_encoder::{JavaScriptEncoder, JavaScriptEncoderMode, JavaScriptEncoderSettings};

    fn generic_tests(encoder: &JavaScriptEncoder) {
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

    fn ascii_only_tests(encoder: &JavaScriptEncoder) {
        let simple_unicode_test = encoder.encode(&'\u{1234}'.to_string());
        assert_eq!("\\u1234", simple_unicode_test);

        let high_ascii_test = encoder.encode(&'\u{ff}'.to_string());
        assert_eq!("\\xff", high_ascii_test);
    }
    #[test]
    fn test_encoder_trait() {
        let encoder = JavaScriptEncoder::create('\\', ValidAsciiRange::ASCII, JavaScriptEncoderSettings {mode: JavaScriptEncoderMode::Block});
        generic_tests(&encoder);
    }
}