import os

file_path = r"c:\Users\jenaf\source\repos\skatsearch\src\lib_wasm.rs"

# Complete logic for update_game_type with identity preservation
new_update_method = r"""    #[wasm_bindgen]
    pub fn update_game_type(&mut self, game_type_str: &str) {
        use crate::skat::context::ProblemTransformation::*;
        
        // 1. Determine New Game Type and Transformation
        let (new_game_type, new_trans) = match game_type_str {
            "Spades" => (Game::Suit, Some(SpadesSwitch)),
            "Hearts" => (Game::Suit, Some(HeartsSwitch)),
            "Diamonds" => (Game::Suit, Some(DiamondsSwitch)),
            "Clubs" => (Game::Suit, None),
            "Grand" => (Game::Grand, None),
            "Null" => (Game::Null, None),
            _ => (Game::Suit, None), 
        };

        // 2. Preserve UI Identity of Cards
        // Helper to apply/unapply trans (XOR swap is its own inverse)
        let apply_trans = |cards: u32, t: Option<crate::skat::context::ProblemTransformation>| -> u32 {
            if let Some(tr) = t {
                GameContext::get_switched_cards(cards, tr)
            } else {
                cards
            }
        };

        // Get current UI cards
        let current_trans = self.active_transformation;
        let ui_declarer = apply_trans(self.engine.context.declarer_cards, current_trans);
        let ui_left = apply_trans(self.engine.context.left_cards, current_trans);
        let ui_right = apply_trans(self.engine.context.right_cards, current_trans);
        let ui_skat = apply_trans(self.skat_cards, current_trans);

        // Calculate new Internal cards
        let new_declarer = apply_trans(ui_declarer, new_trans);
        let new_left = apply_trans(ui_left, new_trans);
        let new_right = apply_trans(ui_right, new_trans);
        let new_skat = apply_trans(ui_skat, new_trans);

        // 3. Update State
        self.engine.context.game_type = new_game_type;
        self.active_transformation = new_trans;
        
        self.engine.context.declarer_cards = new_declarer;
        self.engine.context.left_cards = new_left;
        self.engine.context.right_cards = new_right;
        self.skat_cards = new_skat;

        // Update Position
        self.current_position = self.engine.create_initial_position();
    }
"""

with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

# Locate the existing function update_game_type
start_marker = "    pub fn update_game_type(&mut self, game_type_str: &str) {"
# Include the #[wasm_bindgen] line if present roughly before? No, locate function signature.
start_idx = content.find(start_marker)

if start_idx == -1:
    print("Error: Function start not found")
    exit(1)

# Backtrack to include attribute if present
# Usually #[wasm_bindgen] is line before.
# Lets check simplistic approach: look for `#[wasm_bindgen]` before signature.
pre_idx = content.rfind("#[wasm_bindgen]", 0, start_idx)
if pre_idx != -1 and pre_idx > start_idx - 50: # Close enough
    start_idx = pre_idx

# Locate the end of the function
idx = content.find(start_marker) # Forward again to brace finding
while idx < len(content):
    if content[idx] == '{':
        break
    idx += 1

brace_count = 1
idx += 1
found_end = False
end_idx = -1

while idx < len(content):
    if content[idx] == '{':
        brace_count += 1
    elif content[idx] == '}':
        brace_count -= 1
        if brace_count == 0:
            end_idx = idx + 1
            break
    idx += 1

if end_idx == -1:
    print("Error: Could not find end of function")
    exit(1)

# Check content to confirm it's roughly what we expect (old version)
old_chunk = content[start_idx:end_idx]
print(f"Replacing chunk of length {len(old_chunk)}")

# Replace
new_content = content[:start_idx] + new_update_method + content[end_idx:]

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(new_content)

print("Successfully updated update_game_type in lib_wasm.rs")
