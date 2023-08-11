#[derive(Clone, Copy, Default)]
pub struct SuitData {
    // sort key, ascending
    pub factor: u8,

    // suit
    pub suit_pattern: u32,

    // number of cards for given suit
    pub(crate) cnt_p1: u8,
    pub(crate) cnt_p2: u8,
    pub(crate) cnt_p3: u8,

    // number of highest cards for given suit
    pub ups_p1: u8,
    pub(crate) ups_p2: u8,
    pub(crate) ups_p3: u8
}