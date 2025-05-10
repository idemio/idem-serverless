pub struct EncoderRuleSet {
    valid_chars: Vec<char>,
    char_replace: Vec<(char, &'static str)>,
    valid_bit_mask: u64
}