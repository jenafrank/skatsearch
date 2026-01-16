import os

file_path = r"c:\Users\jenaf\source\repos\skatsearch\src\lib_wasm.rs"

fixed_function = r"""    fn get_sorted_hand_display_string(&self, cards_mask: u32, game_type: Game) -> String {
        use crate::consts::bitboard::*;

        let trans = self.active_transformation;
        let mut sorted_str = Vec::new();

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
            
            // Iterate A..7 (6 down to 0 + offset)
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
        // Order: Spades, Hearts, Diamonds
        // Offsets: Spades=14, Hearts=7, Diamonds=0
        
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

# Locate the function start
start_marker = "    fn get_sorted_hand_display_string(&self, cards_mask: u32, game_type: Game) -> String {"
start_idx = content.find(start_marker)

if start_idx == -1:
    print("Error: Function start not found")
    exit(1)

# Locate the end of the function (counting braces)
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

print("Successfully replaced function.")
