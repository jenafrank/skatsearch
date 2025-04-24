use crate::consts::bitboard::{ALLCARDS, JACKOFCLUBS, JACKOFDIAMONDS, JACKOFHEARTS, JACKOFSPADES};
use crate::traits::{Augen, StringConverter};
use crate::types::counter::Counters;
use crate::types::game::{self, Game};
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

pub fn allgames_battle(number_of_samples: usize) -> std::io::Result<()> {
    let mut file = File::create(r"data_allgames_battle.txt")?;
    
    let allnow = Instant::now();
    let distros = get_random_card_distros(number_of_samples);

    let mut i=0;
    let mut results: Vec<(Ply, Option<WonGame>)> = Vec::new();

    for (player_a_cards, player_b_cards, player_c_cards) in distros {

        println!("GAME {i} -----+-+-+-+-+-++---------------------- ");
        
        let start_pos_a = if i % 3 == 0 { Player::Declarer } else if i % 3 == 1 { Player::Left }     else { Player::Right };
        let start_pos_b = if i % 3 == 0 { Player::Right }    else if i % 3 == 1 { Player::Declarer } else { Player::Left };
        let start_pos_c = if i % 3 == 0 { Player::Left }     else if i % 3 == 1 { Player::Right }    else { Player::Declarer };

        let now = Instant::now();
        let skat = ALLCARDS ^ player_a_cards ^ player_b_cards ^ player_c_cards;        
        
        let res1 = Solver::calc_all_games(player_b_cards, player_c_cards, player_a_cards, start_pos_a);
        let res2 = Solver::calc_all_games(player_c_cards, player_a_cards, player_b_cards, start_pos_b);
        let res3 = Solver::calc_all_games(player_a_cards, player_b_cards, player_c_cards, start_pos_c);

        let mut playerWon: Ply = Ply::NA;        
        let mut won_game: Option<WonGame> = None;
        let mut best_points: u32 = 0;

        println!("\nPlayer A ---------------- ");
        let playerA = print_scorecard(player_a_cards, player_b_cards, player_c_cards, skat, res1);
        match playerA {
            Some(best) => if best.points > best_points {playerWon = Ply::A; won_game = Some(best); best_points = best.points;},
            None => {},
        }

        println!("\nPlayer B ---------------- ");
        let playerB = print_scorecard(player_b_cards, player_c_cards, player_a_cards, skat, res2);
        match playerB {
            Some(best) => if best.points > best_points {playerWon = Ply::B; won_game = Some(best); best_points = best.points;},
            None => {},
        }

        println!("\nPlayer C ---------------- ");
        let playerC = print_scorecard(player_c_cards, player_a_cards, player_b_cards, skat, res3);
        match playerC {
            Some(best) => if best.points > best_points {playerWon = Ply::C; won_game = Some(best); best_points = best.points;},
            None => {},
        }

        match won_game {
            Some(game) => println!("{}: {} | {} - Hand: {}",playerWon, game.points, game.game, game.hand),
            None => println!("EINGEMISCHT"),
        }

        results.push((playerWon, won_game));
        
    }

    // Show all results


    println!("");
    println!(" ------------- FULL RESULTS ----------------- ");
    println!("");

    for (ply, game) in results {
        match game {
            Some(game) => println!("{}: {} | {} - Hand: {}",ply, game.points, game.game, game.hand),
            None => println!("EINGEMISCHT"),
        }
    }

    Ok(())
}

fn print_scorecard(declarer_cards: u32, left_cards: u32, right_cards: u32, skat: u32, res: Result<crate::types::solver::retargs::AllGames, crate::types::solver::allgames::CalculationError>) 
-> Option<WonGame> {
    match res {
        Ok(values) => {
            println!("Declarer  : {}", declarer_cards.__str());
            println!("Left      : {}", left_cards.__str());
            println!("Right     : {}", right_cards.__str());
            println!("Skat      : {}", skat.__str());
            println!("            {:4} | {:4} | {:4} | {:4} | {:4} | {:4} ","Eich","Grue","Herz","Sche","Grnd","Null");
            println!(" Mit Skat : {:4} | {:4} | {:4} | {:4} | {:4} | {:4} ",values.eichel_farbe, values.gruen_farbe, values.herz_farbe, values.schell_farbe, values.grand, values.null);
            println!("     Hand : {:4} | {:4} | {:4} | {:4} | {:4} | {:4} ",values.eichel_hand, values.gruen_hand, values.herz_hand, values.schell_hand, values.grand_hand, values.null_hand);

            let wongames = get_wongames(values, declarer_cards);
            println!(" WonGames: {:4}", wongames.len());

            for game in &wongames {
                println!(" {:}, {:}, {:}, {:} ",game.points, game.game, game.value, game.hand);
            }

            let best: Option<&WonGame> = max_won_game_ref(&wongames);

            match best {
                Some(best_won_game) => {
                    println!("Reizwert: {:4} | Spiel: {:} | Wert: {:}", best_won_game.points, best_won_game.game, best_won_game.value);
                },
                None => {
                    println!("Weg");
                }
            }

            println!("end---");

            return best.copied();
        },
        Err(_) => todo!(),        
    }
}

fn max_won_game_ref(games: &[WonGame]) -> Option<&WonGame> {
    games.iter().max_by_key(|wg| wg.points)
}

#[derive(Clone, Copy)]
pub enum Ply {
    A, B, C, NA
}

impl std::fmt::Display for Ply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {            
            Ply::A => write!(f, " [A] "),
            Ply::B => write!(f, " [B] "),
            Ply::C => write!(f, " [C] "),
            Ply::NA => write!(f, " [NA] "),            
        }
    }
}

#[derive(Clone, Copy)]
pub struct WonGame {
    value: u8,
    points: u32,
    game: WonGameType,
    hand: bool
}

#[derive(Clone, Copy)]
pub enum WonGameType {
    Eichel,
    Gruen,
    Herz,
    Schell,
    Grand,
    Null,
    NA
}

impl std::fmt::Display for WonGameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WonGameType::Eichel => write!(f, "Eichel"),
            WonGameType::Gruen => write!(f, "Gruen"),
            WonGameType::Herz => write!(f, "Herz"),
            WonGameType::Schell => write!(f, "Schell"),
            WonGameType::Grand => write!(f, "Grand"),
            WonGameType::Null => write!(f, "Null"),
            WonGameType::NA => write!(f, "N/A"),
        }
    }
}

fn get_wongames(values: crate::types::solver::retargs::AllGames, cards: u32) -> Vec<WonGame> {

    let mut ret: Vec<WonGame> = Vec::new();

    let games = [
        (values.eichel_farbe, 12, WonGameType::Eichel, false),
        (values.gruen_farbe, 11, WonGameType::Gruen, false),
        (values.herz_farbe, 10, WonGameType::Herz, false),
        (values.schell_farbe, 9,  WonGameType::Schell, false),
        (values.grand, 24,  WonGameType::Grand, false),
        (values.eichel_hand, 12, WonGameType::Eichel, true),
        (values.gruen_hand, 11, WonGameType::Gruen, true),
        (values.herz_hand, 10, WonGameType::Herz, true),
        (values.schell_hand, 9,  WonGameType::Schell, true),
        (values.grand_hand, 24,  WonGameType::Grand, true),        
    ];
    
    for &(value, multiplier, game, hand) in &games {
        // nur weiter, wenn > 60
        if value <= 60 {
            continue;
        }
    
        // Basis‑Faktor
        let mut factor = get_factor(cards) + 1;
    
        // Schneider‑Bonus
        if value > 89 {
            factor += 1;
        }

        // Hand-Bonus
        if hand {
            factor += 1;
        }
    
        // Punkte berechnen und pushen
        let points = factor * multiplier;
        ret.push(WonGame { 
            value, 
            points, 
            game, 
            hand 
        });
    }

    if values.null == 0 {
        ret.push(WonGame { value: 0, points: 23, game: WonGameType::Null, hand: false });
    }

    if values.null_hand == 0 {
        ret.push(WonGame { value: 0, points: 35, game: WonGameType::Null, hand: true });
    }

    ret
}

const JACK_MASKS: [u32; 4] = [
    JACKOFCLUBS,
    JACKOFSPADES,
    JACKOFHEARTS,
    JACKOFDIAMONDS,
];

    
fn get_factor(cards: u32) -> u32 {
    // Ist der Clubs‑Bube gesetzt?
    let first_present = cards & JACKOFCLUBS != 0;

    // Zähle, wie viele Jacks von links anfangen,
    // bei denen (gesetzt? == first_present)
    // true liefern, und stoppe beim ersten Wechsel.
    JACK_MASKS
        .iter()
        .take_while(|&&mask| (cards & mask != 0) == first_present)
        .count() as u32
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
