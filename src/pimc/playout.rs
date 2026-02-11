use super::facts::Facts;
use crate::extensions::solver::solve_all_cards_from_position;
use crate::skat::context::GameContext;
use crate::skat::engine::SkatEngine;

use super::pimc_problem_builder::PimcProblemBuilder;
use super::pimc_search::PimcSearch;
use crate::consts::bitboard::{ACES, EIGHTS, JACKS, KINGS, NINES, QUEENS, TENS};
use crate::skat::defs::{Player, CLUBS, DIAMONDS, HEARTS, SPADES};
use crate::traits::{Bitboard, StringConverter};

fn get_points(card: u32) -> u32 {
    if (card & ACES) != 0 {
        return 11;
    }
    if (card & TENS) != 0 {
        return 10;
    }
    if (card & KINGS) != 0 {
        return 4;
    }
    if (card & QUEENS) != 0 {
        return 3;
    }
    if (card & JACKS) != 0 {
        return 2;
    }
    0
}

fn get_null_rank(card: u32) -> u32 {
    if (card & ACES) != 0 {
        return 7;
    }
    if (card & KINGS) != 0 {
        return 6;
    }
    if (card & QUEENS) != 0 {
        return 5;
    }
    if (card & JACKS) != 0 {
        return 4;
    }
    if (card & TENS) != 0 {
        return 3;
    }
    if (card & NINES) != 0 {
        return 2;
    }
    if (card & EIGHTS) != 0 {
        return 1;
    }
    0 // Sevens
}

pub fn playout(true_context: GameContext, n_samples: u32, god_players: &[Player]) {
    println!("Starting Playout...");

    let mut pos = true_context.create_initial_position();

    let mut facts_declarer = Facts::zero_fact();
    let mut facts_left = Facts::zero_fact();
    let mut facts_right = Facts::zero_fact();

    let mut current_trick: Vec<(Player, u32)> = Vec::new();

    // Reconstruct current_trick from pos.trick_cards if starting mid-trick
    if pos.trick_cards != 0 && current_trick.is_empty() {
        let count = pos.trick_cards.count_ones();
        if count == 1 {
            // Assume 1 card = Previous Player
            let prev_player = match pos.player {
                Player::Declarer => Player::Right,
                Player::Left => Player::Declarer,
                Player::Right => Player::Left,
            };
            // Extract single card
            let (cards, _) = pos.trick_cards.__decompose();
            let card = cards[0];
            current_trick.push((prev_player, card));
            println!(
                "Context: Reconstructed history: {:?} played {}",
                prev_player,
                card.__str()
            );
        }
    }

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

        let stich_num = (round - 1) / 3 + 1;
        let sub_stich = (round - 1) % 3 + 1;

        println!(
            "Stich {}.{} : Turn: {:?} (Cards: {})",
            stich_num,
            sub_stich,
            turn,
            my_cards.__str()
        );
        println!(
            "  Scores -> Decl: {}, Team: {}",
            pos.declarer_points, pos.team_points
        );

        // 1. Determine Move using PIMC

        // Generate PimcProblem for Current Player
        let mut builder = PimcProblemBuilder::new(true_context.game_type())
            .my_player(turn)
            .turn(turn)
            .declarer_start_points(pos.declarer_points)
            .threshold(true_context.threshold_upper);

        builder = builder.cards(turn, &my_cards.__str());

        // Set Remaining Cards: correct logic is 'All unplayed' MINUS 'My Cards'
        // (and minus trick cards, but get_all_unplayed_cards usually includes hands only?
        //  Wait, check Position implementation: yes, declarer_cards | left_cards | right_cards.
        //  So trick cards are NOT in there. Correct.)
        let all_unplayed = pos.get_all_unplayed_cards();
        let remaining = all_unplayed & !my_cards;
        builder = builder.remaining_cards(&remaining.__str());

        // Set Table cards logic
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

        // Log Facts
        println!("  Facts:");
        println!("    Right: {}", facts_right.convert_to_string());

        let result: Vec<(u32, f32)>;
        let mut best_move_card = 0;
        let mut val_debug: f32 = 0.0;

        // GOD MODE CHECK
        if god_players.contains(&turn) {
            println!("  [GOD MODE] Player {:?} has perfect info.", turn);
            // Create temporary engine to solve
            let mut temp_engine = SkatEngine::new(true_context.clone(), None);
            // We use solve_all_cards_from_position
            let solve_res = solve_all_cards_from_position(&mut temp_engine, &pos, 0, 120);

            // The result is Vec<(card, response, value)>
            // We want the BEST card.
            // If Declarer: Max Value.
            // If Defender: Min Value.
            // (Assuming 'Value' is declarer points)

            // Note: 'solve_all_cards' assumes optimal counter-play.
            // So we just pick the card that leads to the best outcome for the current player.

            if solve_res.results.is_empty() {
                println!("  [GOD MODE] No moves found? Panic.");
                break;
            }

            // Sort by value
            let mut moves = solve_res.results.clone();

            if turn == Player::Declarer {
                // Declarer wants to MAXIMIZE points (or win condition)
                // For now assuming Points Game or simple win.
                // Just picking Max Value.
                moves.sort_by(|a, b| b.2.cmp(&a.2)); // Descending
            } else {
                // Opponent wants to MINIMIZE declarer points.
                moves.sort_by(|a, b| a.2.cmp(&b.2)); // Ascending
            }

            best_move_card = moves[0].0;
            let best_val = moves[0].2; // u8
            val_debug = best_val as f32;

            // Mock result for printing
            result = vec![(best_move_card, val_debug)];
        } else {
            // STANDARD PIMC
            let problem = builder.build();

            // 2. Search
            let search = PimcSearch::new(problem, n_samples, None);
            result = search.estimate_probability_of_all_cards(false);

            if !result.is_empty() {
                best_move_card = result[0].0;
                val_debug = result[0].1;
            }
        }

        // Print Analysis
        println!("  Analysis:");
        for (card, val) in &result {
            println!("    Card: {} -> Win Prob: {:.4}", card.__str(), val);
        }

        if result.is_empty() {
            println!("  No valid moves found. (Panic?)");
            break;
        }

        // Chosen above
        let val = val_debug;

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
                println!("  (Inferred: {:?} is void in suit)", turn);
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

pub struct GameTrace {
    pub moves: Vec<u32>,
    pub win_probs: Vec<f32>,
    pub declarer_won: bool,
    pub declarer_points: u8,
}

pub fn playout_with_history(true_context: GameContext, n_samples: u32) -> GameTrace {
    let mut pos = true_context.create_initial_position();

    let mut facts_declarer = Facts::zero_fact();
    let mut facts_left = Facts::zero_fact();
    let mut facts_right = Facts::zero_fact();

    let mut current_trick: Vec<(Player, u32)> = Vec::new();

    // Reconstruct current_trick if starting mid-trick
    if pos.trick_cards != 0 {
        let count = pos.trick_cards.count_ones();
        if count == 1 || count == 2 {
            // Handle mid-trick reconstruction if needed later
        }
    }

    let mut moves_history = Vec::new();
    let mut probs_history = Vec::new();
    let mut declarer_tricks = 0;

    let mut round = 0;

    // Debug Initial Context
    // Debug Initial Context
    // println!(
    //     "Truth: D:{:x} L:{:x} R:{:x} S:{:x}",
    //     true_context.declarer_cards(),
    //     true_context.left_cards(),
    //     true_context.right_cards(),
    //     true_context.get_skat()
    // );

    loop {
        // Check Game Over
        if pos.declarer_cards == 0
            && pos.left_cards == 0
            && pos.right_cards == 0
            && pos.trick_cards == 0
        {
            let won = if true_context.game_type() == crate::skat::defs::Game::Null {
                declarer_tricks == 0
            } else {
                pos.declarer_points >= 61
            };

            return GameTrace {
                moves: moves_history,
                win_probs: probs_history,
                declarer_won: won,
                declarer_points: pos.declarer_points,
            };
        }

        let turn = pos.player;
        let my_cards = pos.player_cards;

        // PIMC Move Selection
        println!("Playout Game Type: {:?}", true_context.game_type());
        let mut builder = PimcProblemBuilder::new(true_context.game_type())
            .my_player(turn)
            .turn(turn)
            .declarer_start_points(pos.declarer_points)
            .threshold(true_context.threshold_upper);

        builder = builder.cards(turn, &my_cards.__str());

        let all_unplayed = pos.get_all_unplayed_cards();
        let skat = true_context.get_skat();
        let remaining = (all_unplayed | skat) & !my_cards;
        println!(
            "Turn: {:?}, My: {}, Unplayed: {}, Skat: {}, Rem: {}",
            turn,
            my_cards.count_ones(),
            all_unplayed.count_ones(),
            skat.count_ones(),
            remaining.count_ones()
        );
        builder = builder.remaining_cards(&remaining.__str());

        // Table cards reconstruction
        let mut prev_card = 0u32;
        let mut next_card = 0u32;

        if pos.trick_cards != 0 {
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

            for &(p, card) in &current_trick {
                if p == prev_player {
                    prev_card = card;
                } else if p == next_player {
                    next_card = card;
                }
            }

            if prev_card != 0 {
                builder = builder.trick_previous_player(pos.trick_suit, prev_card);
            } else {
                builder = builder.active_suit(pos.trick_suit);
            }

            if next_card != 0 {
                builder = builder.trick_next_player(next_card);
            }
        }

        builder = builder.facts(Player::Declarer, facts_declarer);
        builder = builder.facts(Player::Left, facts_left);
        builder = builder.facts(Player::Right, facts_right);

        let problem = builder.build();
        let search = PimcSearch::new(problem, n_samples, None);
        let result = search.estimate_probability_of_all_cards(false);

        let (best_move_card, win_prob) = if !result.is_empty() {
            let mut results = result;
            // Sort by heuristic
            results.sort_by(|(card_a, prob_a), (card_b, prob_b)| {
                // 1. Probability (Descending)
                let prob_cmp = prob_b
                    .partial_cmp(prob_a)
                    .unwrap_or(std::cmp::Ordering::Equal);
                if prob_cmp != std::cmp::Ordering::Equal {
                    return prob_cmp;
                }

                // 2. Safety (Avoid immediate loss) - ONLY for Null games
                // Check if *this* move is safe (doesn't result in Declarer winning trick)
                let check_is_safe = |card: u32| -> bool {
                    // 1. Simulate move
                    let mut sim_pos = pos.clone();
                    sim_pos = sim_pos.make_move(card, &true_context);

                    // 2. If trick finished, check winner
                    if sim_pos.trick_cards == 0 {
                        // Trick finished. Winner is sim_pos.player
                        // Safe if Winner != Declarer.
                        return sim_pos.player != Player::Declarer;
                    }

                    // 3. If trick NOT finished, check if I am currently winning
                    // And if future opponents are FORCED to beat me.
                    // Logic: I am Unsafe (Likely to Win) if I am currently high
                    // AND opponents can avoiding beating me.
                    // I am Safe (Likely to Lose) if I am currently low
                    // OR opponents MUST beat me.

                    // Determine current trick winner so far
                    // sim_pos has updated trick_cards (including my card).
                    // We need to see if 'card' is highest in trick?
                    // Or just ask: Can effectively "Declarer" win this?

                    // If I am NOT Declarer, this logic doesn't matter (PIMC is usually for me).
                    // But playout runs for all.
                    if turn != Player::Declarer {
                        return true; // As defender, winning trick is Good/Safe for Null (usually).
                                     // Wait, defenders want Declarer to win. So Defender winning is BAD?
                                     // But simulation is generally from POV of "My Player".
                                     // WinProb=0 means "My Player" loses.
                                     // If I am Right (Defender), and P=0, means Declarer Wins game.
                                     // So I want to preventing Declarer winning.
                                     // But the Request was specifically about "Alleinspieler" (Declarer).
                                     // "Alleinspieler berechnet sich..."
                                     // So apply this strict safety only for Declarer.
                    }
                    // I am Declarer. I played 'card'.
                    // sim_pos.trick_cards contains my card + previous.
                    // Is 'card' winning?
                    // We need the cards on table.
                    let mut sim_pos = pos.clone();
                    sim_pos = sim_pos.make_move(card, &true_context);

                    // Safe means: The result of the trick is favorable to ME.
                    // Implies: Evaluating the *end* of the trick.
                    // But we are in the middle. We need to Simulate the trick end?
                    // Or just "Does this card WIN currently?"
                    // Original logic: "If I am winning, check if future players force overtake."

                    // Copy logic from above but return bool
                    // Simplified re-simulation of trick end locally:
                    let lead_suit = sim_pos.trick_suit;
                    let (cards, _) = sim_pos.trick_cards.__decompose();

                    let mut winning_card = card;
                    let mut current_winner_rank = get_null_rank(card);

                    // 1. Determine who is winning between Me and Table
                    for &c in &cards {
                        if (c & lead_suit) != 0 && (winning_card & lead_suit) == 0 {
                            // I played off-suit (discard). Table card (suit) beast me.
                            winning_card = c;
                        } else if (c & lead_suit) != 0 && (winning_card & lead_suit) != 0 {
                            // Both suit. Compare rank.
                            if get_null_rank(c) > get_null_rank(winning_card) {
                                winning_card = c;
                            }
                        }
                        // If table is winning, I am Safe (assuming Declarer logic).
                        // If I am winning, I might be Unsafe.
                    }

                    let am_winning = winning_card == card;
                    let is_declarer = turn == crate::skat::defs::Player::Declarer;

                    // Logic for Null:
                    // Declarer wants to NOT win.
                    // Defenders want Declarer to WIN.

                    if is_declarer {
                        if !am_winning {
                            return true;
                        } // Not winning -> Safe
                    } else {
                        // Defender
                        if !am_winning {
                            return true;
                        } // I am not winning. Good for defender (Declarer might win).
                          // If I am winning, check if future players will overtake ME.
                    }

                    // Check future (same as before)
                    // We need 'next_player' relative to 'turn' (the player making the move)
                    // The sim_pos.player here is the player who will play *next* after 'turn' has played 'card'.
                    let next_p = sim_pos.player;

                    let next_hand = crate::skat::rules::get_cards_for_player(
                        true_context.declarer_cards(),
                        true_context.left_cards(),
                        true_context.right_cards(),
                        next_p,
                    );

                    let legal = crate::skat::rules::get_legal_moves(lead_suit, next_hand);
                    let (legal_cards, _) = legal.__decompose();

                    let mut must_overtake = true;
                    for &lc in &legal_cards {
                        if (lc & lead_suit) == 0 {
                            must_overtake = false;
                            break;
                        }
                        if get_null_rank(lc) < current_winner_rank {
                            must_overtake = false;
                            break;
                        }
                    }

                    if must_overtake {
                        return true;
                    } // Future player overtakes. I am safe (Not winning).

                    return false; // I am winning and next player might not overtake. Unsafe/Bad.
                };

                if true_context.game_type() == crate::skat::defs::Game::Null {
                    let safe_a = check_is_safe(*card_a);
                    let safe_b = check_is_safe(*card_b);

                    if safe_a && !safe_b {
                        return std::cmp::Ordering::Less;
                    } // A safe (Better) -> Less
                    if !safe_a && safe_b {
                        return std::cmp::Ordering::Greater;
                    }
                }

                // 3. Card Value
                if true_context.game_type() == crate::skat::defs::Game::Null {
                    let rank_a = get_null_rank(*card_a);
                    let rank_b = get_null_rank(*card_b);

                    if pos.trick_cards == 0 {
                        // LEADING: Smallest Rank is Better (to force opponents to take)
                        // Ascending Sort: A < B. Smallest first.
                        rank_a.cmp(&rank_b)
                    } else {
                        // FOLLOWING:
                        // If Safe (Dump): Highest Rank is Better
                        // If Unsafe (Loss): Smallest Rank is Better (Beauty/Face Saving)

                        let safe_a = check_is_safe(*card_a);
                        // safe_b is same as safe_a here (otherwise filtered earlier)

                        if safe_a {
                            // SAFE: Dump High -> Descending: B < A
                            rank_b.cmp(&rank_a)
                        } else {
                            // UNSAFE: Play Low -> Ascending: A < B
                            rank_a.cmp(&rank_b)
                        }
                    }
                } else {
                    // Suit/Grand: Lower Points = Better (appear earlier)
                    let points_a = get_points(*card_a);
                    let points_b = get_points(*card_b);
                    // Compare A to B (Ascending) -> Smaller points appear earlier.
                    points_a.cmp(&points_b)
                }
            });
            results[0]
        } else {
            let legal = pos.get_legal_moves();
            (legal.__decompose().0[0], 0.5)
        };

        // Record History
        moves_history.push(best_move_card);
        probs_history.push(win_prob);

        // FACT UPDATES
        if pos.trick_cards != 0 {
            let lead_suit = pos.trick_suit;
            // println!(
            //     "Check Infer: Turn {:?} Lead {:x} Move {:x}",
            //     turn, lead_suit, best_move_card
            // );
            if (best_move_card & lead_suit) == 0 {
                // Player couldn't follow suit -> Void
                let mut facts = match turn {
                    Player::Declarer => facts_declarer,
                    Player::Left => facts_left,
                    Player::Right => facts_right,
                };
                if (lead_suit & CLUBS) != 0 {
                    facts.no_clubs = true;
                    // println!("INFER: {:?} No Clubs", turn);
                } else if (lead_suit & SPADES) != 0 {
                    facts.no_spades = true;
                    // println!("INFER: {:?} No Spades", turn);
                } else if (lead_suit & HEARTS) != 0 {
                    facts.no_hearts = true;
                    // println!("INFER: {:?} No Hearts", turn);
                } else if (lead_suit & DIAMONDS) != 0 {
                    facts.no_diamonds = true;
                    // println!("INFER: {:?} No Diamonds", turn);
                }

                match turn {
                    Player::Declarer => facts_declarer = facts,
                    Player::Left => facts_left = facts,
                    Player::Right => facts_right = facts,
                };
            }
        }

        // Track trick
        current_trick.push((turn, best_move_card));

        // Execute Move
        let trick_finished = pos.trick_cards.count_ones() == 2;

        pos = pos.make_move(best_move_card, &true_context);

        if trick_finished {
            current_trick.clear();
            if pos.player == Player::Declarer {
                declarer_tricks += 1;
                if true_context.game_type() == crate::skat::defs::Game::Null {
                    break;
                }
            }
        }

        round += 1;
        if round > 40 {
            break;
        }
    }

    GameTrace {
        moves: moves_history,
        win_probs: probs_history,
        declarer_won: if true_context.game_type() == crate::skat::defs::Game::Null {
            declarer_tricks == 0
        } else {
            pos.declarer_points >= 61
        },
        declarer_points: pos.declarer_points,
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
        playout(up.generate_concrete_problem(), 20, &[]);
    }
}
