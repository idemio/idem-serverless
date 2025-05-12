use crate::common::{HEX, HEX_MASK, HEX_SHIFT, char_bucket, char_mask};
use fnv::FnvHashMap;

pub struct EncodeOption {
    pub(crate) min_len: bool,
    pub(crate) simple_escape: bool,
}

pub struct CompiledEncoderRules {
    pub(crate) valid_ascii_mask: [u32; 8],
    pub(crate) replace_map: FnvHashMap<char, &'static str>,
    pub(crate) encode_map: FnvHashMap<char, EncodeOption>,
    pub(crate) output_buffer_max_len: usize,
}

pub enum ValidAsciiRange {
    ASCII,
    ASCIIExtended,
    NoRestrict,
}

#[derive(Clone, Debug)]
pub enum RuleType {
    Esc { min: bool, simple: bool },
    Sub(&'static str),
}

#[derive(Clone, Debug)]
pub enum Rule {
    Range {
        start_character: char,
        end_character: char,
        exclude: Option<Vec<char>>,
        rule_type: RuleType,
    },
    Uni {
        ch: char,
        rt: RuleType,
    },
}

pub trait Encoder<C>
where
    C: Sized,
{
    fn create(escape_char: char, valid_ascii_range: ValidAsciiRange, config: C) -> Self;
    fn compile_encoder_rules<const B: usize>(rules: [Rule; B]) -> CompiledEncoderRules {
        let mut valid_mask = [u32::MAX; 8];
        let mut replace_chars: FnvHashMap<char, &'static str> = FnvHashMap::default();
        let mut encode_chars: FnvHashMap<char, EncodeOption> = FnvHashMap::default();
        let mut largest_replace = 1usize;
        for rule in rules {
            match rule {
                Rule::Range {
                    start_character: start,
                    end_character: end,
                    exclude,
                    rule_type,
                } => {
                    assert!((start as u32) < (end as u32));

                    // all ascii and extended ascii
                    if (start as u32) < 256 && (end as u32) < 256 {
                        match rule_type {
                            RuleType::Esc {
                                simple: simple_escape,
                                min: min_len,
                            } => {
                                for char in start..end {
                                    if exclude.as_ref().is_some_and(|excl| excl.contains(&char)) {
                                        continue;
                                    }
                                    let bucket: usize = char_bucket(char);
                                    let char_mask: u32 = !char_mask(char);
                                    valid_mask[bucket] &= char_mask;
                                    encode_chars.insert(
                                        char,
                                        EncodeOption {
                                            min_len,
                                            simple_escape,
                                        },
                                    );
                                }
                            }
                            RuleType::Sub(replace_string) => {
                                for char in start..end {
                                    if exclude.as_ref().is_some_and(|excl| excl.contains(&char)) {
                                        continue;
                                    }
                                    largest_replace = largest_replace.max(replace_string.len());
                                    let bucket: usize = char_bucket(char);
                                    let char_mask: u32 = !char_mask(char);
                                    valid_mask[bucket] &= char_mask;
                                    replace_chars.insert(char, replace_string);
                                }
                            }
                        }
                    } else {
                        match rule_type {
                            RuleType::Esc {
                                simple: simple_escape,
                                min: min_len,
                            } => {
                                for char in start..end {
                                    if exclude.as_ref().is_some_and(|excl| excl.contains(&char)) {
                                        continue;
                                    }
                                    encode_chars.insert(
                                        char,
                                        EncodeOption {
                                            min_len,
                                            simple_escape,
                                        },
                                    );
                                }
                            }
                            RuleType::Sub(replace_string) => {
                                for char in start..end {
                                    if exclude.as_ref().is_some_and(|excl| excl.contains(&char)) {
                                        continue;
                                    }
                                    largest_replace = largest_replace.max(replace_string.len());
                                    replace_chars.insert(char, replace_string);
                                }
                            }
                        }
                    }
                }
                Rule::Uni {
                    ch: c,
                    rt: rule_type,
                } => {
                    if (c as u32) <= 256 {
                        match rule_type {
                            RuleType::Esc {
                                min: min_len,
                                simple: simple_escape,
                            } => {
                                let bucket: usize = char_bucket(c);
                                let char_mask: u32 = !char_mask(c);
                                valid_mask[bucket] &= char_mask;
                                encode_chars.insert(
                                    c,
                                    EncodeOption {
                                        min_len,
                                        simple_escape,
                                    },
                                );
                            }
                            RuleType::Sub(replace_string) => {
                                largest_replace = largest_replace.max(replace_string.len());
                                let bucket: usize = char_bucket(c);
                                let char_mask: u32 = !char_mask(c);
                                valid_mask[bucket] &= char_mask;
                                replace_chars.insert(c, replace_string);
                            }
                        }
                    } else {
                        match rule_type {
                            RuleType::Esc {
                                min: min_len,
                                simple: simple_escape,
                            } => {
                                encode_chars.insert(
                                    c,
                                    EncodeOption {
                                        min_len,
                                        simple_escape,
                                    },
                                );
                            }
                            RuleType::Sub(replace_string) => {
                                largest_replace = largest_replace.max(replace_string.len());
                                replace_chars.insert(c, replace_string);
                            }
                        }
                    }
                }
            }
        }

        CompiledEncoderRules {
            valid_ascii_mask: valid_mask,
            replace_map: replace_chars,
            encode_map: encode_chars,
            output_buffer_max_len: largest_replace,
        }
    }
    fn compiled_rules(&self) -> &CompiledEncoderRules;
    fn escape_char(&self) -> char;
    fn ascii_properties(&self) -> &ValidAsciiRange;

    fn encode(&self, input: &str) -> String {
        let mut output =
            String::with_capacity(self.compiled_rules().output_buffer_max_len * input.len());

        for character in input.chars() {
            // Fast route with ascii mask
            if (character as u32) < 256 {
                let bucket = char_bucket(character);
                let mask = char_mask(character);

                // Check if there is a rule defined for the character
                if self.compiled_rules().valid_ascii_mask[bucket] & mask == 0 {
                    if self.compiled_rules().encode_map.contains_key(&character) {
                        let encode_option =
                            self.compiled_rules().encode_map.get(&character).unwrap();

                        if encode_option.simple_escape {
                            output.push(self.escape_char());
                            output.push(character);
                            continue;
                        }

                        if character as u32 <= 0xFF {
                            Self::encode_as_hex_byte(self.escape_char(), &mut output, character);
                            continue;
                        }

                        Self::encode_as_unicode(self.escape_char(), &mut output, character);
                        continue;
                    }
                } else {
                    match self.ascii_properties() {
                        ValidAsciiRange::ASCII => {
                            if (character as u32) < 127 {
                                output.push(character);
                                continue;
                            }
                        }
                        _ => {}
                    }
                }
            }
            if self.compiled_rules().replace_map.contains_key(&character) {
                let replace = self.compiled_rules().replace_map.get(&character).unwrap();
                output.push_str(replace);
                continue;
            }
            match self.ascii_properties() {
                ValidAsciiRange::NoRestrict => {
                    output.push(character);
                }
                _ => {
                    if character as u32 <= 0xFF {
                        Self::encode_as_hex_byte(self.escape_char(), &mut output, character);
                    } else {
                        Self::encode_as_unicode(self.escape_char(), &mut output, character);
                    }
                }
            }
        }
        output.shrink_to_fit();
        output
    }

    // Helper methods to extract repeated code
    #[inline]
    fn encode_as_hex_byte(escape_char: char, output: &mut String, character: char) {
        output.reserve(4);
        output.push(escape_char);
        output.push('x');
        output.push(HEX[(character as u32 >> HEX_SHIFT) as usize]);
        output.push(HEX[(character as u32 & HEX_MASK) as usize]);
    }

    #[inline]
    fn encode_as_unicode(escape_char: char, output: &mut String, character: char) {
        output.reserve(6);
        output.push(escape_char);
        output.push('u');
        output.push(HEX[(character as u32 >> (3 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
        output.push(HEX[(character as u32 >> (2 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
        output.push(HEX[(character as u32 >> (1 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
        output.push(HEX[(character as u32 & HEX_MASK) as usize]);
    }
}

//macro_rules! uni_rule {
//    ($ch:expr, sub $text:expr) => {
//        Rule::Uni { ch: $ch, rt: RuleType::Sub($text) }
//    };
//
//    ($ch:expr, esc $simple:expr, $min:expr) => {
//        Rule::Uni { ch: $ch, rt: RuleType::Esc { simple: $simple, min: $min } }
//    };
//}