use crate::extensions::solver::SolveAllCardsRet;
use crate::skat::counters::Counters;
use crate::skat::defs::Player;
use crate::skat::engine::SkatEngine;
use std::time::Instant;

#[derive(Default)]
pub struct PlayoutLine {
    pub declarer_cards: u32,
    pub left_cards: u32,
    pub right_cards: u32,
    pub player: Player,
    pub card: u32,
    pub declarer_points: u8,
    pub team_points: u8,
    pub cnt_iters: usize,
    pub cnt_breaks: usize,
    pub time: u128,
}

#[derive(Default)]
pub struct PlayoutAllCardsRetLine {
    pub player: Player,
    pub best_card: u32,
    pub declarer_points: u8,
    pub all_cards: SolveAllCardsRet,
}

pub fn playout(engine: &mut SkatEngine) -> Vec<PlayoutLine> {
    let mut ret: Vec<PlayoutLine> = Vec::new();
    let mut i: usize = 0;
    // engine.context.number_of_cards()? GameContext doesn't have it?
    // GameContext has cards. total cards = 30? or count?
    let _n = 30; // Standard Skat game has 30 cards in play (32 total - 2 skat).
                 // Or check if incomplete game?
                 // engine.context.declarer_cards.count_ones() * 3?
                 // Let's assume standard game or calculate.
    let n = (engine.context.declarer_cards.count_ones()
        + engine.context.left_cards.count_ones()
        + engine.context.right_cards.count_ones()) as usize;

    let mut position = engine.create_initial_position();

    while i < n {
        let mut row: PlayoutLine = Default::default();

        row.declarer_cards = position.declarer_cards;
        row.left_cards = position.left_cards;
        row.right_cards = position.right_cards;

        let mut cnt: Counters = Counters::new();

        let now = Instant::now();
        let alpha = 0;
        let beta = 120;
        let (played_card, _) = engine.search(&position, &mut cnt, alpha, beta);
        let time = now.elapsed().as_millis();

        row.player = position.player; // Position has player? Yes.
        row.card = played_card;
        row.declarer_points = position.declarer_points;
        row.team_points = position.team_points;
        row.cnt_iters = cnt.iters as usize;
        row.cnt_breaks = cnt.breaks as usize;
        row.time = time;

        position = position.make_move(played_card, &engine.context);

        ret.push(row);
        i += 1;
    }

    ret
}

pub fn playout_all_cards(engine: &mut SkatEngine) -> Vec<PlayoutAllCardsRetLine> {
    let mut ret: Vec<PlayoutAllCardsRetLine> = Vec::new();
    let mut i: usize = 0;
    // let time = 0; // Only supporting 1 game in playout for now? Or loop?
    // let n = engine.context.get_player_cards(engine.context.start_player).count_ones();
    //     + engine.context.left_cards.count_ones()
    //     + engine.context.right_cards.count_ones()) as usize;

    let n = (engine.context.declarer_cards.count_ones()
        + engine.context.left_cards.count_ones()
        + engine.context.right_cards.count_ones()) as usize;

    let mut position = engine.create_initial_position();

    while i < n {
        // Assuming n cards to play.
        let mut row: PlayoutAllCardsRetLine = Default::default();

        let mut cnt: Counters = Counters::new();

        // solve_classic logic: search
        let (best_card, _) = engine.search(&position, &mut cnt, 0, 120);

        // get_all_cards logic
        let mut results = Vec::new();
        let moves_word = position.get_legal_moves();
        let (moves, num_moves) = crate::skat::rules::get_sorted_by_value(moves_word);

        for mov in &moves[0..num_moves] {
            let child_pos = position.make_move(*mov, &engine.context);
            // Search child
            let (best_response, val) = engine.search(&child_pos, &mut cnt, 0, 120);
            results.push((*mov, best_response, val));
        }
        let resall = SolveAllCardsRet { results };

        row.player = position.player;
        row.best_card = best_card;

        position = position.make_move(best_card, &engine.context);

        row.declarer_points = position.declarer_points;
        row.all_cards = resall;

        ret.push(row);

        i += 1;
    }

    ret
}

impl Default for SolveAllCardsRet {
    fn default() -> Self {
        SolveAllCardsRet {
            results: Vec::new(),
        }
    }
}
