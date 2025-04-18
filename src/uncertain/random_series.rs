use crate::consts::bitboard::ALLCARDS;
use crate::traits::{Augen, StringConverter};
use crate::types::counter::Counters;
use crate::types::game::Game;
use crate::types::player::Player;
use crate::types::problem::Problem;
use crate::types::solver::withskat::acceleration_mode::AccelerationMode;
use crate::types::solver::Solver;
use crate::types::state::State;
use crate::types::tt_table::TtTable;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::cmp::max;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
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
        let mut cnt = Counters::new();

        let result = p.search(&s, &mut tt, &mut cnt);

        println!(
            "{:5} ms {:9} iters {:3} pnts | D: {} L: {} R: {}",
            now.elapsed().as_millis(),
            cnt.iters,
            result.1 + p.get_skat().__get_value(),
            cards.0.__str(),
            cards.1.__str(),
            cards.2.__str()
        );

        file.write_fmt(format_args!(
            "{} {} {} \n",
            now.elapsed().as_millis(),
            cnt.iters,
            result.1 + p.get_skat().__get_value()
        ))?;
    }

    Ok(())
}

pub fn sample_farbe_declarer_tt_dd_parallel(number_of_samples: usize) -> std::io::Result<()> {
    // Datei und Zähler in einen Arc/Mutex packen, um den gemeinsamen Zugriff zu schützen.
    let file = Arc::new(Mutex::new(File::create("data_par.txt")?));
    let total_wins = Arc::new(AtomicU32::new(0));
    let allnow = Instant::now();
    let distros = get_random_card_distros(number_of_samples);

    // Parallel Iteration: Hier wird der Range in einen Paralleliterator umgewandelt.
    distros.into_par_iter().for_each(|(declarer_cards, left_cards, right_cards)| {        
        
        let now = Instant::now();
        let result = Solver::solve_with_skat(
            left_cards, 
            right_cards, 
            declarer_cards, 
            Game::Grand, 
            Player::Declarer,
        AccelerationMode::AlphaBetaAccelerating);

        // Ermitteln Sie den besten Wert aus dem Ergebnis.
        let best_value = result.all_skats
            .iter()
            .map(|card| card.value)
            .max()
            .unwrap_or(0);

        // Erhöhen Sie den Gewinnzähler, falls der Wert eine bestimmte Grenze überschreitet.
        if best_value >= 61 {
            total_wins.fetch_add(1, Ordering::SeqCst);
        }
        let wins = total_wins.load(Ordering::SeqCst);

        // Formatieren Sie den Fortschritts-String.
        let progress_line = format!(
            "{:2} -- {:8} ms - {:5} ms {:9} | {:9} iters/colls - {:7.2} {:6} pnts | D: {} L: {} R: {}",
            wins,
            allnow.elapsed().as_millis(),
            now.elapsed().as_millis(),
            result.counters.iters,
            result.counters.collisions,
            (result.counters.collisions as f32) / (result.counters.iters as f32) * 1000.0,
            best_value,
            declarer_cards.__str(),
            left_cards.__str(),
            right_cards.__str()
        );

        // Direkte Ausgabe auf dem Bildschirm. Hier können die Ausgaben
        // unübersichtlich werden, wenn mehrere Threads gleichzeitig schreiben.
        println!("{}", progress_line);

        // Schreiben in die Datei – schützen Sie den Zugriff mit einem Mutex.
        {
            let mut file = file.lock().unwrap();
            writeln!(
                file,
                "{:2} , {:8} , {:5} , {:9} , {:9} , {:7.2} , {:6} , {} , {} , {}",
                wins,
                allnow.elapsed().as_millis(),
                now.elapsed().as_millis(),
                result.counters.iters,
                result.counters.collisions,
                (result.counters.collisions as f32) / (result.counters.iters as f32) * 1000.0,
                best_value,
                declarer_cards.__str(),
                left_cards.__str(),
                right_cards.__str()
            ).unwrap();
        }
    });

    Ok(())
}

pub fn sample_farbe_declarer_tt_dd(number_of_samples: usize) -> std::io::Result<()> {
    let mut file = File::create(r"data.txt")?;
    
    let mut total_wins:u32 = 0;
    let allnow = Instant::now();
    let distros = get_random_card_distros(number_of_samples);

    for (declarer_cards, left_cards, right_cards) in distros {
        
        let now = Instant::now();                        
        let result = Solver::solve_with_skat(
            left_cards, 
            right_cards, 
            declarer_cards, 
            Game::Grand, 
            Player::Declarer, 
            AccelerationMode::WinningOnly);       
        
        let mut best_value = 0;
        for card in result.all_skats {
            best_value = max(best_value, card.value);
        }        

        total_wins += if best_value >= 61 { 1 } else { 0 };

        println!(
            "{:2} -- {:8} ms - {:5} ms {:9} | {:9} iters/colls - {:7.2} {:6} pnts | D: {} L: {} R: {}",
            total_wins,
            allnow.elapsed().as_millis(),
            now.elapsed().as_millis(),
            result.counters.iters,
            result.counters.collisions,
            (result.counters.collisions as f32)/(result.counters.iters as f32)*1000.,
            best_value,
            // result.best_skat.unwrap().value,
            declarer_cards.__str(),
            left_cards.__str(),
            right_cards.__str()
        );

        file.write_fmt(format_args!(
            "{:2} , {:8} , {:5} , {:9}, {:9}, {:7.2}, {:6}, {}, {}, {} \n",
            total_wins,
            allnow.elapsed().as_millis(),
            now.elapsed().as_millis(),
            result.counters.iters,
            result.counters.collisions,
            (result.counters.collisions as f32)/(result.counters.iters as f32)*1000.,
            best_value,
            declarer_cards.__str(),
            left_cards.__str(),
            right_cards.__str()
        ))?;
    }

    Ok(())
}

pub fn allgames(number_of_samples: usize) -> std::io::Result<()> {
    let mut file = File::create(r"data_allgames.txt")?;
    
    let allnow = Instant::now();
    let distros = get_random_card_distros(number_of_samples);

    for (declarer_cards, left_cards, right_cards) in distros {
        
        let now = Instant::now();
        let skat = ALLCARDS ^ declarer_cards ^ left_cards ^ right_cards;
        let res = Solver::calc_all_games(left_cards, right_cards, declarer_cards, Player::Declarer);

        match res {
            Ok(values) => {
                println!("Declarer  : {}", declarer_cards.__str());
                println!("Left      : {}", left_cards.__str());
                println!("Right     : {}", right_cards.__str());
                println!("Skat      : {}", skat.__str());
                println!("            {:4} | {:4} | {:4} | {:4} | {:4} | {:4} ","Eich","Grue","Herz","Sche","Grnd","Null");
                println!(" Mit Skat : {:4} | {:4} | {:4} | {:4} | {:4} | {:4} ",values.eichel_farbe, values.gruen_farbe, values.herz_farbe, values.schell_farbe, values.grand, values.null);
                println!("     Hand : {:4} | {:4} | {:4} | {:4} | {:4} | {:4} ",values.eichel_hand, values.gruen_hand, values.herz_hand, values.schell_hand, values.grand_hand, values.null_hand);
            },
            Err(_) => todo!(),
        }
    }

    Ok(())
}

fn get_random_card_distros(number_of_distros: usize) -> Vec<(u32, u32, u32)> {
    let mut rand = StdRng::seed_from_u64(223);
    let mut ret = Vec::<(u32, u32, u32)>::new();

    for _ in 0..number_of_distros {
        let cards = get_random_card_distribution_with_seed(&mut rand);   
        ret.push(cards);
    }

    ret    
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
