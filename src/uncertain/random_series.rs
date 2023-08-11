use std::fs::File;
use std::io::Write;
use std::time::Instant;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{SeedableRng, thread_rng};
use crate::traits::{Augen, StringConverter};
use crate::types::game::Game;
use crate::types::player::Player;
use crate::types::problem::Problem;
use crate::types::problem_transposition_table::ProblemTranspositionTable;

pub fn sample_farbe_declarer_tt(number_of_samples: usize) -> std::io::Result<()> {

    let mut file = File::create(r"C:\Users\m1fpeuke\PyCharmProjects\p1\data\data3.txt")?;
    let mut rand = StdRng::seed_from_u64(222);

    for i in 0..number_of_samples {
        let cards = get_random_card_distribution_with_seed(&mut rand);
        let p = Problem::create(
            cards.0, cards.1, cards.2,
            Game::Grand, Player::Declarer);

        let (mut ptt, stt)= ProblemTranspositionTable::from_problem_with_initial_state(p);

        let now = Instant::now();
        let result = ptt.search(&stt);

        println!("{:5} ms {:9} iters {:3} pnts | D: {} L: {} R: {}",
                 now.elapsed().as_millis(),
                 ptt.counters.cnt_iters,
                 result.1 + p.get_skat().__get_value(),
                 cards.0.__str(),
                 cards.1.__str(),
                 cards.2.__str());

        file.write_fmt(format_args!("{} {} {} \n", now.elapsed().as_millis(), ptt.counters.cnt_iters, result.1 + p.get_skat().__get_value()))?;
    }

    Ok(())
}

pub fn sample_farbe_declarer_tt_dd(number_of_samples: usize) -> std::io::Result<()> {

    let mut file = File::create(r"C:\Users\m1fpeuke\PyCharmProjects\p1\data\data3.txt")?;
    let mut rand = StdRng::seed_from_u64(222);

    for i in 0..number_of_samples {
        let cards = get_random_card_distribution_with_seed(&mut rand);
        let p = Problem::create(
            cards.0, cards.1, cards.2,
            Game::Grand, Player::Declarer);

        let now = Instant::now();
        let result = ProblemTranspositionTable::search_win_loss(p);

        match p.game_type {
            Game::Null => {
                println!("{:5} ms {:9} | {:9} iters/colls {:3} pnts | D: {} L: {} R: {}",
                         now.elapsed().as_millis(),
                         result.1,
                         result.2,
                         result.0,
                         cards.0.__str(),
                         cards.1.__str(),
                         cards.2.__str());

                file.write_fmt(format_args!("{} {} {} \n", now.elapsed().as_millis(), result.1, result.0))?;
            }
            _ => {
                println!("{:5} ms {:9} | {:9} iters/colls {:3} pnts | D: {} L: {} R: {}",
                         now.elapsed().as_millis(),
                         result.1,
                         result.2,
                         result.0 + p.get_skat().__get_value(),
                         cards.0.__str(),
                         cards.1.__str(),
                         cards.2.__str());

                file.write_fmt(format_args!("{} {} {} \n", now.elapsed().as_millis(), result.1, result.0 + p.get_skat().__get_value()))?;
            }
        }

    }

    Ok(())
}

pub fn get_random_card_distribution_with_seed(rand: &mut StdRng) -> (u32, u32, u32) {
    let mut vec : Vec<usize> = (0..32).collect();

    // vec.shuffle(&mut thread_rng());
    vec.shuffle(rand);

    let mut declarer_cards = 0u32;
    let mut left_cards = 0u32;
    let mut right_cards = 0u32;

    for el in &vec[0..10] {
        declarer_cards |= 1u32 << *el;
    }

    for el in &vec[10..20] {
        left_cards |= 1u32 << *el;
    }

    for el in &vec[20..30] {
        right_cards |= 1u32 << *el;
    }

    (declarer_cards, left_cards, right_cards)
}

#[cfg(test)]
mod tests {
    use crate::traits::{Bitboard, StringConverter};
    use crate::uncertain::random_series::get_random_card_distribution;

    #[test]
    pub fn test1() {
        let x = get_random_card_distribution();

        println!("D: {}", x.0.__str());
        println!("L: {}", x.1.__str());
        println!("R: {}", x.2.__str());
    }

}