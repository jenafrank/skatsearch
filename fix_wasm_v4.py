import os

file_path = r"c:\Users\jenaf\source\repos\skatsearch\src\lib_wasm.rs"

# Complete logic for get_sorted_hand_display_string
new_method = r"""    fn get_sorted_hand_display_string(&self, cards_mask: u32, game_type: Game) -> String {
        use crate::consts::bitboard::*;
        use crate::skat::context::ProblemTransformation::*;

        let trans = self.active_transformation;
        let mut sorted_str = Vec::new();

        if game_type == Game::Null {
             // Null Sort: Suits (C, S, H, D) -> Ranks (A, K, Q, J, 10, 9, 8, 7)
            // Iterate Suits: Clubs, Spades, Hearts, Diamonds
            let suits_data = [
                (JACKOFCLUBS, 21),   // Clubs
                (JACKOFSPADES, 14),  // Spades
                (JACKOFHEARTS, 7),   // Hearts
                (JACKOFDIAMONDS, 0), // Diamonds
            ];

            for (jack_mask, offset) in suits_data.iter() {
                // Rank Order for Null: A(6), K(4), Q(3), J, 10(5), 9(2), 8(1), 7(0)
                let rank_offsets = [
                    (6, "A"),
                    (4, "K"),
                    (3, "Q"),
                    (99, "J"), // Pseudo offset for Jack
                    (5, "10"),
                    (2, "9"),
                    (1, "8"),
                    (0, "7"),
                ];

                for (r_off, _) in rank_offsets.iter() {
                    let bit = if *r_off == 99 {
                        *jack_mask
                    } else {
                        1 << (offset + r_off)
                    };

                    if (cards_mask & bit) != 0 {
                        // Null: identity preservation ensures we just display the bitstring
                        sorted_str.push(bit.__str().trim().to_string());
                    }
                }
            }
        } else {
            // Standard Suit/Grand Sorting
            // 1. Jacks (Fixed Order: C, S, H, D -> Best to Worst)
            // Jacks are always Trump.
            let jacks = [JACKOFCLUBS, JACKOFSPADES, JACKOFHEARTS, JACKOFDIAMONDS];
            for &j in &jacks {
                // Check if internal cards contain this UI Jack
                // Internal = trans(UI)
                let internal_bit = if let Some(t) = trans {
                    GameContext::get_switched_cards(j, t)
                } else {
                    j
                };
                
                if (cards_mask & internal_bit) != 0 {
                     // Display UI card
                     sorted_str.push(j.__str().trim().to_string());
                }
            }

            // Define UI Suits Order: Clubs, Spades, Hearts, Diamonds
            // Offsets for the 7 card (base of the suit)
            let ui_suits = [
               (21, "Clubs"),
               (14, "Spades"),
               (7, "Hearts"),
               (0, "Diamonds")
            ];
            
            // Determine UI Trump Suit Offset (if any)
            let ui_trump_offset = match (game_type, trans) {
                (Game::Grand, _) => None,
                (Game::Suit, None) => Some(21), // Clubs
                (Game::Suit, Some(SpadesSwitch)) => Some(14),
                (Game::Suit, Some(HeartsSwitch)) => Some(7),
                (Game::Suit, Some(DiamondsSwitch)) => Some(0),
                _ => Some(21), // Fallback
            };

            // Helper to iterate a suit's cards (A..7, skipping Jacks)
            let mut process_suit = |base_offset: u8| {
                 // Ranks: A(6) .. 7(0). Jacks handled separately.
                 for r in (0..7).rev() {
                      let ui_bit = 1 << (base_offset + r);
                      let internal_bit = if let Some(t) = trans {
                          GameContext::get_switched_cards(ui_bit, t)
                      } else {
                          ui_bit
                      };
                      
                      if (cards_mask & internal_bit) != 0 {
                          sorted_str.push(ui_bit.__str().trim().to_string());
                      }
                 }
            };

            // 2. Trump Suit Body (if applicable)
            if let Some(trump_off) = ui_trump_offset {
                process_suit(trump_off);
            }

            // 3. Other Suits (in fixed C-S-H-D order)
            for &(suit_off, _) in &ui_suits {
                // Skip if it is the trump suit
                if let Some(trump_off) = ui_trump_offset {
                    if suit_off == trump_off {
                        continue;
                    }
                }
                process_suit(suit_off);
            }
        }
        
        // Return space-separated
        sorted_str.join(" ")
    }
"""

with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

# Locate start
start_marker = "fn get_sorted_hand_display_string(&self, cards_mask: u32, game_type: Game) -> String {"
start_idx = content.find(start_marker)

if start_idx == -1:
    print("Error: Function start not found")
    exit(1)

# Locate the corrupted end sequence
# 746:             for &j in &jacks {
# 747:         res.push(']');
# 748:         res
# 749:     }

corrupted_end_marker = "res.push(']');"
end_marker_idx = content.find(corrupted_end_marker, start_idx)

if end_marker_idx == -1:
    print("Error: Corrupted end marker not found")
    exit(1)

# Find the closing brace after that
end_idx = content.find("}", end_marker_idx) + 1

if end_idx == 0: # -1 + 1 = 0
    print("Error: Could not find end brace")
    exit(1)

print(f"Replacing corrupted chunk from {start_idx} to {end_idx}")
new_content = content[:start_idx] + new_method + content[end_idx:]

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(new_content)

print("Successfully patched corrupted get_sorted_hand_display_string")
