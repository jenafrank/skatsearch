# Skat Search Engine - CLI Documentation

This document provides a comprehensive overview of the `skat_aug23` Command Line Interface (CLI). It explains the available commands, their purpose, and provides usage examples.

## Building the Project

To build the tool, run:

```bash
cargo build --release
```

The executable will be located at `target/release/skat_aug23.exe`.

## Common Arguments

Many commands use common flags:
- `--context <FILE>`: Path to a JSON file defining the game state (cards, active player, etc.).
- `--samples <N>`: Number of Monte Carlo samples to run (for PIMC commands). Higher is more accurate but slower.
- `--log-file <PATH>`: Optional path to write detailed logs.

## Commands Overview

The CLI functionality is divided into several categories:

### 1. Mass Simulation & Research
Tools for analyzing general game probabilities by simulating thousands of random hands.

#### `analyze-grand`
Analyzes Grand Hand scenarios. Generates random hands and simulates optimal play to determine win rates based on hand features (Jacks, Aces, etc.).

**Usage:**
```bash
# Analyze 1000 random hands (played as Hand game)
skat_aug23 analyze-grand --count 1000 --hand

# Analyze with Skat Pickup (optimizes discard)
skat_aug23 analyze-grand --count 100 --samples 50

# Analyze post-discard (simulates having already discarded optimally)
skat_aug23 analyze-grand --count 100 --post-discard
```
**Output:** CSV data to stdout (or file via `--output`).

#### `analyze-suit`
Analyzes Suit Game scenarios (defaults to Clubs). Similar to `analyze-grand` but for Suit games.
**New:** Includes detailed progress logging and incremental CSV saving.

**Usage:**
```bash
# Analyze 500 Suit hands with Skat pickup
skat_aug23 analyze-suit --count 500 --samples 50 --output results.csv
```

#### `generate-json`
Searches for specific interesting scenarios (e.g., winning Grand hands with 1 Jack and 2 Aces) and saves them as JSON files for further debugging.

**Usage:**
```bash
skat_aug23 generate-json --count 5 --min-win 0.9 --output-dir scenarios/
```

---

### 2. Game Type & Bidding Helper
Tools to help decide *what* to play.

#### `best-game`
Determines the optimum game contract (Grand, Suit, Null) for a given 12-card hand using **Perfect Information** (assumes you know all cards).
*Useful for theoretical maximum score.*

**Usage:**
```bash
skat_aug23 best-game --context hand_12_cards.json --mode best
```

#### `pimc-best-game`
Determines the best game contract using **PIMC (Incomplete Information)**. This simulates realistic play where opponents' cards are unknown.
*Closest to a real player's decision.*

**Usage:**
```bash
skat_aug23 pimc-best-game --context hand_12_cards.json --samples 50
```

#### `skat-calc`
Evaluates the best Skat discard for a 12-card hand. Iterates all 2-card discards to find the highest value game.

**Usage:**
```bash
skat_aug23 skat-calc --context hand_12_cards.json --mode best
```

---

### 3. Single Game Analysis (PIMC)
Tools to analyze a specific game state with incomplete information.

#### `pimc-calc`
Calculates the value of a specific game state using PIMC.
- `win`: Win Probability (0.0 - 1.0).
- `best`: Returns the best move and its value.

**Usage:**
```bash
skat_aug23 pimc-calc --context game_state.json --mode win --samples 100
```

#### `playout`
Simulates a game from the current state to the end using PIMC. Shows the likely sequence of moves.

**Usage:**
```bash
skat_aug23 playout --context game.json --samples 20
```

---

### 4. Perfect Information Solver
Tools that solve the game exactly (using Alpha-Beta pruning), assuming all cards are known.

#### `value-calc`
Calculates the exact game value (points).

**Usage:**
```bash
skat_aug23 value-calc --context game_state.json
```

#### `analysis`
Performs a single-step analysis, evaluating *every legal move* available to the current player and its exact value.

**Usage:**
```bash
skat_aug23 analysis --context game_state.json
```

#### `standard-playout`
Plays out the rest of the game assuming perfect play.

**Usage:**
```bash
skat_aug23 standard-playout --context game.json
```

## Input File Format (Context JSON)

Most commands require a JSON context file.
**Example (`game.json`):**
```json
{
  "game_type": "Grand",
  "start_player": "Declarer",
  "my_cards": "SJ,HA,HK,DA,DK,CQ,C9,S9,H9,D9", 
  "declarer_cards": "...", // If solving with perfect info
  "left_cards": "...",
  "right_cards": "...",
  "mode": "Win" // Or "Value"
}
```
*Note: For PIMC commands, you typically provide `my_cards` and `remaining_cards` or allow the tool to distribute the rest.*
