pub(crate) const HEX_SHIFT: u32 = 4;
pub(crate) const HEX_MASK: u32 = 0x0F;
pub(crate) const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

#[inline]
pub(crate) const fn char_mask(c: char) -> u32 {
    1 << (c as u32 & 31)
}

#[inline]
pub(crate) const fn char_bucket(c: char) -> usize {
    (c as u32 >> 5) as usize
}