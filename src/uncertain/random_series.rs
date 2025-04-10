use crate::traits::{Augen, StringConverter};
use crate::types::counter::Counters;
use crate::types::game::Game;
use crate::types::player::Player;
use crate::types::problem::Problem;
use crate::types::solver::Solver;
use crate::types::state::State;
use crate::types::tt_table::TtTable;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::cmp::max;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

pub fn sample_farbe_declarer_tt(number_of_samples: usize) -> std::io::Result<()> {
    let mut file = File::create(r"C:\Users\m1fpeuke\PyCharmProjects\p1\data\data3.txt")?;
    let mut rand = StdRng::seed_from_u64(222);

    for _ in 0..number_of_samples {
        let cards = get_random_card_distribution_with_seed(&mut rand);
        let p = Problem::create(cards.0, cards.1, cards.2, Game::Grand, Player::Declarer);

        let s = State::create_initial_state_from_problem(&p);

        let now = Instant::now();
        let mut tt = TtTable::new();

        let result = p.search(&s, &mut tt);
        let counters = Counters::get();

        println!(
            "{:5} ms {:9} iters {:3} pnts | D: {} L: {} R: {}",
            now.elapsed().as_millis(),
            counters.iters,
            result.1 + p.get_skat().__get_value(),
            cards.0.__str(),
            cards.1.__str(),
            cards.2.__str()
        );

        file.write_fmt(format_args!(
            "{} {} {} \n",
            now.elapsed().as_millis(),
            counters.iters,
            result.1 + p.get_skat().__get_value()
        ))?;
    }

    Ok(())
}

pub fn sample_farbe_declarer_tt_dd(number_of_samples: usize) -> std::io::Result<()> {
    let mut file = File::create(r"data.txt")?;
    let mut rand = StdRng::seed_from_u64(223);
    let mut total_wins:u32 = 0;
    let allnow = Instant::now();

    for _ in 0..number_of_samples {
        let cards = get_random_card_distribution_with_seed(&mut rand);        
        let now = Instant::now();        
        Counters::reset();
        
        // let p = Problem::create(cards.0, cards.1, cards.2, Game::Grand, Player::Declarer);
        // let mut solver = Solver::new(p);
        // let result = solver.solve_with_skat(true, true);
        
        let result = Solver::solve_with_skat_parallel_brute_force(cards.1, cards.2, cards.0, Game::Grand, Player::Declarer);

        let mut best_value = 0;
        for card in result.all_skats {
            best_value = max(best_value, card.value);
        }        

        total_wins += if best_value >= 61 { 1 } else { 0 };

        println!(
            "{} -- {:8} ms - {:5} ms {:9} | {:9} iters/colls - {:7.2} {:6} pnts | D: {} L: {} R: {}",
            total_wins,
            allnow.elapsed().as_millis(),
            now.elapsed().as_millis(),
            result.counters.iters,
            result.counters.collisions,
            (result.counters.collisions as f32)/(result.counters.iters as f32)*1000.,
            best_value,
            cards.0.__str(),
            cards.1.__str(),
            cards.2.__str()
        );

        file.write_fmt(format_args!(
            "{} , {:8} , {:5} , {:9}, {:9}, {:7.2}, {:6}, {}, {}, {} \n",
            total_wins,
            allnow.elapsed().as_millis(),
            now.elapsed().as_millis(),
            result.counters.iters,
            result.counters.collisions,
            (result.counters.collisions as f32)/(result.counters.iters as f32)*1000.,
            best_value,
            cards.0.__str(),
            cards.1.__str(),
            cards.2.__str()
        ))?;
    }

    Ok(())
}

pub fn get_random_card_distribution_with_seed(rand: &mut StdRng) -> (u32, u32, u32) {
    let mut vec: Vec<usize> = (0..32).collect();

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
    use crate::traits::StringConverter;
    use crate::uncertain::random_series::get_random_card_distribution_with_seed;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    pub fn test1() {
        let mut rand = StdRng::seed_from_u64(222);
        let x = get_random_card_distribution_with_seed(&mut rand);

        println!("D: {}", x.0.__str());
        println!("L: {}", x.1.__str());
        println!("R: {}", x.2.__str());
    }
}
