use crate::encoder::{CompiledEncoderRules, Encoder, Rule, RuleType, ValidAsciiRange};

pub enum XmlEncoderMode {
    All,
    Content,
    Attribute,
    SingleQuotedAttribute,
    DoubleQuotedAttribute
}

pub struct XmlEncoderSettings {
    mode: XmlEncoderMode
}

pub struct XmlEncoder {
    pub(crate) rules: CompiledEncoderRules,
    pub(crate) escape_char: char,
    pub(crate) invalid_char: char,
    pub(crate) ascii_properties: ValidAsciiRange
}

impl Encoder<XmlEncoderSettings> for XmlEncoder {
    fn create(escape_char: char, valid_ascii_range: ValidAsciiRange, config: XmlEncoderSettings) -> Self {
        match config.mode {
            XmlEncoderMode::All => {
                let rules = [
                    Rule::Uni {
                        ch: '&',
                        rt: RuleType::Sub("&amp;")
                    },
                    Rule::Uni {
                        ch: '<',
                        rt: RuleType::Sub("&lt;")
                    },
                    Rule::Uni {
                        ch: '>',
                        rt: RuleType::Sub("&gt;")
                    },
                    Rule::Uni {
                        ch: '\'',
                        rt: RuleType::Sub("&#39;")
                    },
                    Rule::Uni {
                        ch: '"',
                        rt: RuleType::Sub("&#34;")
                    },

                    // TODO - check on invalid char replacement for XML/HTML rules
                ];
                let compiled_rules = Self::compile_encoder_rules(rules);
                XmlEncoder {
                    rules: compiled_rules,
                    escape_char,
                    invalid_char: ' ',
                    ascii_properties: valid_ascii_range
                }
            }
            XmlEncoderMode::Content => todo!(),
            XmlEncoderMode::Attribute => todo!(),
            XmlEncoderMode::SingleQuotedAttribute => todo!(),
            XmlEncoderMode::DoubleQuotedAttribute => todo!(),
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