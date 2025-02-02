use crate::types::player::Player;

#[derive(Default, Copy, Clone)]
pub struct PlayoutLine {
    pub declarer_cards: u32,
    pub left_cards: u32,
    pub right_cards: u32,

    pub player: Player,
    pub card: u32,
    pub augen_declarer: u8,
    pub augen_team: u8,
    pub cnt_iters: u32,
    pub cnt_breaks: u32,

    pub time: u128
}
