use super::facts::Facts;
use crate::skat::context::GameContext;

use super::pimc_problem_builder::PimcProblemBuilder;
use super::pimc_search::PimcSearch;
use crate::skat::defs::{Player, CLUBS, DIAMONDS, HEARTS, SPADES};
use crate::traits::StringConverter;

pub fn playout(true_context: GameContext, n_samples: u32) {
    println!("Starting Playout...");

    // 2. Create Initial Position
    let mut pos = true_context.create_initial_position();

    let mut facts_declarer = Facts::zero_fact();
    let mut facts_left = Facts::zero_fact();
    let mut facts_right = Facts::zero_fact();

    let mut current_trick: Vec<(Player, u32)> = Vec::new();
    let mut round = 1;

    loop {
        let turn = pos.player;
        let my_cards = pos.player_cards; // Cards of current player

        // Check Game Over
        if pos.declarer_cards == 0
            && pos.left_cards == 0
            && pos.right_cards == 0
            && pos.trick_cards == 0
        {
            println!("--------------------------------------------------");
            println!("Game Over.");
            println!("Declarer Points: {}", pos.declarer_points);
            println!("Team Points: {}", pos.team_points);
            println!("--------------------------------------------------");
            break;
        }

        println!(
            "Round {}: Turn: {:?} (Cards: {})",
            round,
            turn,
            my_cards.__str()
        );

        // 1. Determine Move using PIMC

        // Generate PimcProblem for Current Player
        let mut builder = PimcProblemBuilder::new(true_context.game_type())
            .my_player(turn)
            .turn(turn)
            .threshold(true_context.threshold_upper);

        builder = builder.cards(turn, &my_cards.__str());

        // Set Remaining Cards: correct logic is 'All unplayed' MINUS 'My Cards'
        // (and minus trick cards, but get_all_unplayed_cards usually includes hands only?
        //  Wait, check Position implementation: yes, declarer_cards | left_cards | right_cards.
        //  So trick cards are NOT in there. Correct.)
        let all_unplayed = pos.get_all_unplayed_cards();
        let remaining = all_unplayed & !my_cards;
        builder = builder.remaining_cards(&remaining.__str());

        // Set Table Cards logic
        let mut prev_card = 0u32;
        let mut next_card = 0u32;

        if pos.trick_cards != 0 {
            // Identify cards from current_trick history
            let prev_player = match turn {
                Player::Declarer => Player::Right,
                Player::Left => Player::Declarer,
                Player::Right => Player::Left,
            };
            let next_player = match turn {
                Player::Declarer => Player::Left,
                Player::Left => Player::Right,
                Player::Right => Player::Declarer,
            };

            // Note: current_trick contains (Player, Card).
            for &(p, card) in &current_trick {
                if p == prev_player {
                    prev_card = card;
                } else if p == next_player {
                    next_card = card;
                }
            }

            // Apply to builder
            // Note: trick_previous_player implies the card played by the player *before* me.
            // trick_next_player implies the card played by the player *after* me (if they played before me? No).
            // Wait. In standard Skat order:
            // If I am 2nd: 1st played (prev). 3rd hasn't.
            // If I am 3rd: 1st played (next?), 2nd played (prev).
            // Let's resolve 'previous' and 'next' relative to ME.
            // PimcProblemBuilder::trick_previous_player(card) -> sets card for (my_player - 1)
            // PimcProblemBuilder::trick_next_player(card) -> sets card for (my_player + 1)

            if prev_card != 0 {
                // If I am D, Prev is R.
                builder = builder.trick_previous_player(pos.trick_suit, prev_card);
            }
            if next_card != 0 {
                // If I am D, Next is L.
                // If L has played, it means I am the last player (3rd).
                builder = builder.trick_next_player(next_card);
            }
        }

        // Set Facts
        builder = builder.facts(Player::Declarer, facts_declarer);
        builder = builder.facts(Player::Left, facts_left);
        builder = builder.facts(Player::Right, facts_right);

        let problem = builder.build();

        // 2. Search
        let search = PimcSearch::new(problem, n_samples, None);
        let result = search.estimate_probability_of_all_cards(false);

        // Print Analysis
        println!("  Analysis:");
        for (card, val) in &result {
            println!("    Card: {} -> Win Prob: {:.4}", card.__str(), val);
        }

        if result.is_empty() {
            println!("  No valid moves found. (Panic?)");
            break;
        }

        // Choose best move (highest win prob)
        let best_move_card = result[0].0;
        let val = result[0].1;

        println!(
            "  => Player {:?} plays {} (val: {:.4})\n",
            turn,
            best_move_card.__str(),
            val
        );

        // 3. Execution (Apply Move) & Inference

        // Inference Logic: Check if player failed to follow suit
        if pos.trick_cards != 0 {
            let lead_suit = pos.trick_suit;
            if (best_move_card & lead_suit) == 0 {
                // Player void in lead_suit
                let mut facts = match turn {
                    Player::Declarer => facts_declarer,
                    Player::Left => facts_left,
                    Player::Right => facts_right,
                };

                let trump = true_context.game_type().get_trump();

                if (lead_suit & trump) != 0 {
                    facts.no_trump = true;
                } else if (lead_suit & CLUBS) != 0 {
                    facts.no_clubs = true;
                } else if (lead_suit & SPADES) != 0 {
                    facts.no_spades = true;
                } else if (lead_suit & HEARTS) != 0 {
                    facts.no_hearts = true;
                } else if (lead_suit & DIAMONDS) != 0 {
                    facts.no_diamonds = true;
                }

                match turn {
                    Player::Declarer => facts_declarer = facts,
                    Player::Left => facts_left = facts,
                    Player::Right => facts_right = facts,
                };
                // println!("  (Inferred: {:?} is void in suit)", turn);
            }
        }

        // Apply Move
        current_trick.push((turn, best_move_card));

        // Check if trick will be cleared by this move
        if pos.trick_cards_count == 2 {
            current_trick.clear();
        }

        pos = pos.make_move(best_move_card, &true_context);

        if round > 35 {
            break;
        }
        round += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::playout;
    use crate::{pimc::pimc_problem_builder::PimcProblemBuilder, skat::defs::Player};

    #[ignore]
    #[test]
    pub fn test() {
        let up = PimcProblemBuilder::new_farbspiel()
            .cards(Player::Declarer, "CJ SJ D7")
            .remaining_cards("HJ DJ DA DT H7 H8")
            .threshold_half()
            .build();

        // Need to create a game context from problem to playout
        // But PimcProblem is partial.
        // We use generate_concrete_problem to get a starting 'True' context.
        playout(up.generate_concrete_problem(), 20);
    }
}
