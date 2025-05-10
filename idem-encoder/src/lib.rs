mod css_encoder;
mod html_encoder;
mod encoder;
mod xml_encoder;

fn get_character_mask(c: char) -> u32 {
    1 << (c as u32 & 31)
}

// Constants for hex encoding
const HEX_SHIFT: u32 = 4;
const HEX_MASK: u32 = 0x0F;
const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
const LINE_SEPARATOR: char = '\u{2028}';
const PARAGRAPH_SEPARATOR: char = '\u{2029}';

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Source,
    Block,
    Html,
    Attribute,
}

struct JavaScriptEncoder {
    mode: Mode,
    ascii_only: bool,
    valid_masks: [u32; 4],
    hex_encode_quotes: bool,
}

impl JavaScriptEncoder {
    fn new(mode: Mode, ascii_only: bool) -> Self {
        let mut valid_masks = [
            0,
            u32::MAX & !(get_character_mask('\'') | get_character_mask('"')),
            u32::MAX & !get_character_mask('\\'),
            if ascii_only {
                u32::MAX & !get_character_mask(127 as char)
            } else {
                u32::MAX
            },
        ];
        // For BLOCK or HTML mode, also escape '/' and '-'
        if mode == Mode::Block || mode == Mode::Html {
            valid_masks[1] &= !(get_character_mask('/') | get_character_mask('-'));
        }
        // For all modes except SOURCE, escape '&'
        if mode != Mode::Source {
            valid_masks[1] &= !get_character_mask('&');
        }

        let hex_encode_quotes = mode == Mode::Attribute || mode == Mode::Html;
        JavaScriptEncoder {
            mode,
            ascii_only,
            valid_masks,
            hex_encode_quotes,
        }
    }

    fn encode(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len() * 6);
        for c in input.chars() {
            if c as u32 <= 127 {
                let mask_index = c as u32 >> 5;
                let character_mask = get_character_mask(c);
                if (self.valid_masks[mask_index as usize] & character_mask) == 0 {
                    match c {
                        '\u{0008}' => {
                            result.push_str("\\b");
                            continue;
                        }
                        '\u{0009}' => {
                            result.push_str("\\t");
                            continue;
                        }
                        '\u{000a}' => {
                            result.push_str("\\n");
                            continue;
                        }
                        '\u{000c}' => {
                            result.push_str("\\f");
                            continue;
                        }
                        '\u{000d}' => {
                            result.push_str("\\r");
                            continue;
                        }
                        '\'' | '"' => {
                            if self.hex_encode_quotes {
                                result.push('\\');
                                result.push('x');
                                result.push(HEX[(c as u32 >> HEX_SHIFT) as usize]);
                                result.push(HEX[(c as u32 & HEX_MASK) as usize]);
                                continue;
                            } else {
                                // Backslash escape quotes
                                result.push('\\');
                                result.push(c);
                                continue;
                            }
                        }
                        _ => {
                            result.push('\\');
                            result.push('x');
                            result.push(HEX[(c as u32 >> HEX_SHIFT) as usize]);
                            result.push(HEX[(c as u32 & HEX_MASK) as usize]);
                            continue;
                        }
                    }
                }
            } else if self.ascii_only || c == LINE_SEPARATOR || c == PARAGRAPH_SEPARATOR {
                if c as u32 <= 0xFF {
                    result.push('\\');
                    result.push('x');
                    result.push(HEX[(c as u32 >> HEX_SHIFT) as usize]);
                    result.push(HEX[(c as u32 & HEX_MASK) as usize]);
                    continue;
                } else {
                    result.push('\\');
                    result.push('u');

                    // 3
                    result.push(HEX[(c as u32 >> (3 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
                    // 2
                    result.push(HEX[(c as u32 >> (2 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
                    // 1
                    result.push(HEX[(c as u32 >> (1 * HEX_SHIFT)) as usize & HEX_MASK as usize]);
                    // 0
                    result.push(HEX[(c as u32 & HEX_MASK) as usize]);
                    continue;
                }
            }
            result.push(c);
        }

        result.shrink_to_fit();
        result
    }
}

#[cfg(test)]
mod test {
    use crate::{JavaScriptEncoder, Mode};


    ////////////////////////////////////////////////////////////////////////////
    // Java Script Encoder Tests Start
    ////////////////////////////////////////////////////////////////////////////
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
        assert_eq!("\\u2028", line_separator_test);

        let paragraph_separator_test = encoder.encode(&'\u{2029}'.to_string());
        assert_eq!("\\u2029", paragraph_separator_test);

        let simple_lower_case_test = encoder.encode(&"abcd".to_string());
        assert_eq!("abcd", simple_lower_case_test);

        let simple_upper_case_test = encoder.encode(&"ABCD".to_string());
        assert_eq!("ABCD", simple_upper_case_test);
    }

    fn not_ascii_only_test(encoder: &JavaScriptEncoder) {
        let simple_unicode_test = encoder.encode(&'\u{1234}'.to_string());
        assert_eq!("\u{1234}", simple_unicode_test);

        let high_ascii_test = encoder.encode(&'\u{00ff}'.to_string());
        assert_eq!("\u{00ff}", high_ascii_test);
    }

    fn ascii_only_test(encoder: &JavaScriptEncoder) {
        let simple_unicode_test = encoder.encode(&'\u{1234}'.to_string());
        assert_eq!("\\u1234", simple_unicode_test);

        let high_ascii_test = encoder.encode(&'\u{00ff}'.to_string());
        assert_eq!("\\xff", high_ascii_test);
    }

    #[test]
    fn java_script_encode_block_test() {
        let encoder = JavaScriptEncoder::new(Mode::Block, false);
        generic_tests(&encoder);
        not_ascii_only_test(&encoder);

        let double_quote_test = encoder.encode(&'"'.to_string());
        assert_eq!("\\\"", double_quote_test);

        let single_quote_test = encoder.encode(&'\''.to_string());
        assert_eq!("\\\'", single_quote_test);

        let encoder = JavaScriptEncoder::new(Mode::Block, true);
        generic_tests(&encoder);
        ascii_only_test(&encoder);

        let double_quote_test = encoder.encode(&'"'.to_string());
        assert_eq!("\\\"", double_quote_test);

        let single_quote_test = encoder.encode(&'\''.to_string());
        assert_eq!("\\\'", single_quote_test);
    }

    #[test]
    fn java_script_encode_html_test() {
        let encoder = JavaScriptEncoder::new(Mode::Html, false);
        generic_tests(&encoder);
        not_ascii_only_test(&encoder);

        let double_quote_test = encoder.encode(&'"'.to_string());
        assert_eq!("\\x22", double_quote_test);

        let single_quote_test = encoder.encode(&'\''.to_string());
        assert_eq!("\\x27", single_quote_test);

        let encoder = JavaScriptEncoder::new(Mode::Html, true);
        generic_tests(&encoder);
        ascii_only_test(&encoder);

        let double_quote_test = encoder.encode(&'"'.to_string());
        assert_eq!("\\x22", double_quote_test);

        let single_quote_test = encoder.encode(&'\''.to_string());
        assert_eq!("\\x27", single_quote_test);

    }

    #[test]
    fn java_script_encode_source_test() {
        let encoder = JavaScriptEncoder::new(Mode::Source, false);
        generic_tests(&encoder);
        not_ascii_only_test(&encoder);

        let double_quote_test = encoder.encode(&'"'.to_string());
        assert_eq!("\\\"", double_quote_test);

        let single_quote_test = encoder.encode(&'\''.to_string());
        assert_eq!("\\\'", single_quote_test);

        let encoder = JavaScriptEncoder::new(Mode::Source, true);
        generic_tests(&encoder);
        ascii_only_test(&encoder);

        let double_quote_test = encoder.encode(&'"'.to_string());
        assert_eq!("\\\"", double_quote_test);

        let single_quote_test = encoder.encode(&'\''.to_string());
        assert_eq!("\\\'", single_quote_test);
    }

    #[test]
    fn java_script_encode_attribute_test() {
        let encoder = JavaScriptEncoder::new(Mode::Attribute, false);
        generic_tests(&encoder);
        not_ascii_only_test(&encoder);

        let double_quote_test = encoder.encode(&'"'.to_string());
        assert_eq!("\\x22", double_quote_test);

        let single_quote_test = encoder.encode(&'\''.to_string());
        assert_eq!("\\x27", single_quote_test);

        let encoder = JavaScriptEncoder::new(Mode::Attribute, true);
        generic_tests(&encoder);
        ascii_only_test(&encoder);

        let double_quote_test = encoder.encode(&'"'.to_string());
        assert_eq!("\\x22", double_quote_test);

        let single_quote_test = encoder.encode(&'\''.to_string());
        assert_eq!("\\x27", single_quote_test);

    }

    ////////////////////////////////////////////////////////////////////////////
    // HTML Encoder Tests Start
    ////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////
    // XML Encoder Tests Start
    ////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////
    // CSS Encoder Tests Start
    ////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////
    // URI Encoder Tests Start
    ////////////////////////////////////////////////////////////////////////////
}
