use crate::types::{player::Player, counter::Counters};

pub struct SolveRet {
    pub best_card: u32,
    pub best_value: u8,
    pub counters: Counters
}

pub struct SolveWinRet {
    pub best_card: u32,
    pub declarer_wins: bool,
    pub counters: Counters
}

pub struct SolveAllCardsRet {
    pub card_list: Vec<SolveAllLineRetArgs>,
    pub counters: Counters
}

impl Default for SolveAllCardsRet {
    fn default() -> Self {
        Self { 
            card_list: Default::default(), 
            counters: Default::default() 
        }
    }
}

#[derive(Clone, Copy)]
pub struct SolveAllLineRetArgs {
    pub investigated_card: u32,
    pub best_follow_up_card: u32,
    pub value: u8
}

#[derive(Clone, Copy)]
pub struct SolveWithSkatRetLine {
    pub skat_card_1: u32,
    pub skat_card_2: u32,
    pub value: u8
}

pub struct SolveWithSkatRet {
    pub best_skat: Option<SolveWithSkatRetLine>,
    pub all_skats: Vec<SolveWithSkatRetLine>,
    pub counters: Counters
}

pub struct PlayoutAllCardsRetLine {
    pub best_card: u32,
    pub player: Player,
    pub augen_declarer: u8,
    pub all_cards: SolveAllCardsRet
}

impl Default for PlayoutAllCardsRetLine {
    fn default() -> Self {
        Self {
            best_card: Default::default(),
            player: Default::default(),
            augen_declarer: Default::default(),
            all_cards: Default::default(),
        }
     }
}

pub struct AllGames {
    pub eichel_farbe: u8,
    pub gruen_farbe: u8,
    pub herz_farbe: u8,
    pub schell_farbe: u8,
    pub eichel_hand: u8,
    pub gruen_hand: u8,
    pub herz_hand: u8,
    pub schell_hand: u8,

    pub grand: u8,
    pub grand_hand: u8,

    pub null: u8,
    pub null_hand: u8
}
impl AllGames {
    pub(crate) fn default() -> AllGames {
        AllGames {
            eichel_farbe: 123,
            gruen_farbe: 123,
            herz_farbe: 123,
            schell_farbe: 123,
            eichel_hand: 123,
            gruen_hand: 123,
            herz_hand: 123,
            schell_hand: 123,
            grand: 123,
            grand_hand: 123,
            null: 123,
            null_hand: 123,
        }
    }
}
