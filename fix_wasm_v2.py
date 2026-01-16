import os

file_path = r"c:\Users\jenaf\source\repos\skatsearch\src\lib_wasm.rs"

# New method to insert
update_game_type_method = r"""    #[wasm_bindgen]
    pub fn update_game_type(&mut self, game_type_str: &str) {
        use crate::skat::context::ProblemTransformation::*;
        match game_type_str {
            "Spades" => {
                self.engine.context.game_type = Game::Suit;
                self.active_transformation = Some(SpadesSwitch);
            },
            "Hearts" => {
                self.engine.context.game_type = Game::Suit;
                self.active_transformation = Some(HeartsSwitch);
            },
            "Diamonds" => {
                self.engine.context.game_type = Game::Suit;
                self.active_transformation = Some(DiamondsSwitch);
            },
            "Clubs" => {
                self.engine.context.game_type = Game::Suit;
                self.active_transformation = None;
            },
            "Grand" => {
                self.engine.context.game_type = Game::Grand;
                self.active_transformation = None;
            },
            "Null" => {
                self.engine.context.game_type = Game::Null;
                self.active_transformation = None;
            },
            _ => { // Default or Unknown
                // Keep as is
            }
        }
    }
"""

fixed_function = r"""    fn get_sorted_hand_display_string(&self, cards_mask: u32, game_type: Game) -> String {
        use crate::consts::bitboard::*;

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
                // Rank Order: A(6), K(4), Q(3), J, 10(5), 9(2), 8(1), 7(0)
                let rank_offsets = [
                    (6, "A"), 
                    (4, "K"), 
                    (3, "Q"), 
                    (99, "J"), // Pseudo offset for Jack
                    (5, "10"), 
                    (2, "9"), 
                    (1, "8"), 
                    (0, "7")  
                ];

                for (r_off, _) in rank_offsets.iter() {
                    let bit = if *r_off == 99 {
                        *jack_mask
                    } else {
                        1 << (offset + r_off)
                    };

                    if (cards_mask & bit) != 0 {
                        // For Null, NO transposition logic. Identity.
                        sorted_str.push(bit.__str().trim().to_string());
                    }
                }
            }
        } else {
            // Standard Suit/Grand Sorting
            // 1. Jacks (Fixed Order: C, S, H, D -> Best to Worst)
            let jacks = [JACKOFCLUBS, JACKOFSPADES, JACKOFHEARTS, JACKOFDIAMONDS];
            for &j in &jacks {
                if (cards_mask & j) != 0 {
                    let ui_card = if let Some(t) = trans {
                        GameContext::get_switched_cards(j, t)
                    } else {
                        j
                    };
                    sorted_str.push(ui_card.__str().trim().to_string());
                }
            }
    
            // 2. Trump Suit (Non-Jacks)
            if game_type == Game::Suit {
                // Internal Trump is always CLUBS (Bits 21..27)
                let clubs_offset = 21;
                for i in (0..7).rev() {
                    let bit = 1 << (clubs_offset + i);
                    if (cards_mask & bit) != 0 {
                        let ui_card = if let Some(t) = trans {
                            GameContext::get_switched_cards(bit, t)
                        } else {
                            bit
                        };
                        sorted_str.push(ui_card.__str().trim().to_string());
                    }
                }
            }
    
            // 3. Other Suits
            // Mapped order based on UI: Spades, Hearts, Diamonds
            let other_suits_offsets = [14, 7, 0]; 
            
            for &offset in &other_suits_offsets {
                 for i in (0..7).rev() {
                     let bit = 1 << (offset + i);
                     if (cards_mask & bit) != 0 {
                         let ui_card = if let Some(t) = trans {
                            GameContext::get_switched_cards(bit, t)
                        } else {
                            bit
                        };
                        sorted_str.push(ui_card.__str().trim().to_string());
                     }
                 }
            }
        }

        let mut res = String::new();
        res.push('[');
        for (i, s) in sorted_str.iter().enumerate() {
            if i > 0 {
                res.push(' ');
            }
            res.push_str(s);
        }
        res.push(']');
        res
    }"""

with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Insert update_game_type method if not exists
if "pub fn update_game_type" not in content:
    # Insert it before get_sorted_hand_display_string to keep it clean, 
    # or just inside `impl SkatGame` at a safe spot.
    # Let's target the position just before ` fn get_sorted_hand_display_string`
    start_marker = "    fn get_sorted_hand_display_string(&self, cards_mask: u32, game_type: Game) -> String {"
    idx = content.find(start_marker)
    if idx != -1:
        content = content[:idx] + update_game_type_method + "\n" + content[idx:]
    else:
        print("Error: Could not find insertion point for update_game_type")
        exit(1)

# 2. Replace get_sorted_hand_display_string
# Find start again (it might have shifted)
start_marker = "    fn get_sorted_hand_display_string(&self, cards_mask: u32, game_type: Game) -> String {"
start_idx = content.find(start_marker)

if start_idx == -1:
    print("Error: Function start not found")
    exit(1)

# Locate the end of the function
idx = start_idx
brace_count = 0
found_start = False
end_idx = -1

while idx < len(content):
    if content[idx] == '{':
        brace_count += 1
        found_start = True
    elif content[idx] == '}':
        brace_count -= 1
        if found_start and brace_count == 0:
            end_idx = idx + 1
            break
    idx += 1

if end_idx == -1:
    print("Error: Could not find end of function")
    exit(1)

# Replace
new_content = content[:start_idx] + fixed_function + content[end_idx:]

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(new_content)

print("Successfully updated lib_wasm.rs")
