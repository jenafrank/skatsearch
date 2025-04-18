use crate::types::{game::Game, player::Player, problem::Problem};
use super::{retargs::AllGames, withskat::acceleration_mode::AccelerationMode, Solver};

#[derive(Clone, Copy)]
pub enum ProblemTransformation {
    SpadesSwitch,
    HeartsSwitch,
    DiamondsSwitch
}

impl Solver {

    pub fn calc_all_games(left_cards: u32, right_cards: u32, declarer_cards: u32, start_player: Player) -> AllGames {        
        
        let farbe_eichel = Solver::solve_with_skat(left_cards, right_cards, declarer_cards, Game::Farbe, start_player, AccelerationMode::AlphaBetaAccelerating);
        
        let p1 = Problem::create(declarer_cards, left_cards, right_cards, Game::Farbe, start_player);
        let mut solver1 = Solver::new(p1, None);
        let hand_farbe_eichel = solver1.solve();

        let grand = Solver::solve_with_skat(left_cards, right_cards, declarer_cards, Game::Grand, start_player, AccelerationMode::AlphaBetaAccelerating);

        let p2 = Problem::create(declarer_cards, left_cards, right_cards, Game::Grand, start_player);
        let mut solver2 = Solver::new(p2, None);
        let hand_grand = solver2.solve();

        let null = Solver::solve_with_skat(left_cards, right_cards, declarer_cards, Game::Null, start_player, AccelerationMode::AlphaBetaAccelerating);
        
        let p3 = Problem::create(declarer_cards, left_cards, right_cards, Game::Null, start_player);
        let mut solver3 = Solver::new(p3, None);
        let hand_null = solver3.solve_and_add_skat();

        let p4: Problem = Problem::create_transformation(p1, ProblemTransformation::SpadesSwitch);
        let mut solver4 = Solver::new(p4, None);
        let hand_farbe_gruen = solver4.solve_and_add_skat();
        let farbe_gruen = Solver::solve_with_skat(p4.left_cards(), p4.right_cards(), p4.declarer_cards(), Game::Farbe, start_player, AccelerationMode::AlphaBetaAccelerating);

        let p5: Problem = Problem::create_transformation(p1, ProblemTransformation::HeartsSwitch);
        let mut solver5 = Solver::new(p5, None);
        let hand_farbe_herz = solver5.solve_and_add_skat();
        let farbe_herz = Solver::solve_with_skat(p5.left_cards(), p5.right_cards(), p5.declarer_cards(), Game::Farbe, start_player, AccelerationMode::AlphaBetaAccelerating);

        let p6: Problem = Problem::create_transformation(p1, ProblemTransformation::DiamondsSwitch);
        let mut solver6 = Solver::new(p6, None);
        let hand_farbe_schell = solver6.solve_and_add_skat();
        let farbe_schell = Solver::solve_with_skat(p6.left_cards(), p6.right_cards(), p6.declarer_cards(), Game::Farbe, start_player, AccelerationMode::AlphaBetaAccelerating);
                  
        AllGames {
            eichel_farbe: farbe_eichel.best_skat.unwrap().value,
            gruen_farbe: farbe_gruen.best_skat.unwrap().value,
            herz_farbe: farbe_herz.best_skat.unwrap().value,
            schell_farbe: farbe_schell.best_skat.unwrap().value,
            eichel_hand: hand_farbe_eichel.best_value,
            gruen_hand: hand_farbe_gruen.best_value,
            herz_hand: hand_farbe_herz.best_value,
            schell_hand: hand_farbe_schell.best_value,
            grand: grand.best_skat.unwrap().value,
            grand_hand: hand_grand.best_value,
            null: null.best_skat.unwrap().value,
            null_hand: hand_null.best_value,
        }
        
    }

}