use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use skat_aug23::skat::defs::{Game, Player};

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Skat Search Engine - Analysis and Playout Tool

SYNOPSIS:
    skat_aug23 <COMMAND> --context <JSON_FILE> [OPTIONS]

EXAMPLES:
    # Calculate exact game value
    skat_aug23 value-calc --context game_state.json

    # Find best skat discard
    skat_aug23 skat-calc --context hand_12_cards.json --mode best

    # Analyze win/loss for best game type
    skat_aug23 best-game --context hand_12_cards.json --mode win"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Calculates the exact value of the game state using the perfect information solver (Alpha-Beta pruning).
    /// returns the score for the declarer assuming optimal play from all sides. Use this to check if a specific state is won or lost.
    ValueCalc {
        /// Path to the JSON context file
        #[arg(short, long)]
        context: String,
    },
    /// Simulates the game using Perfect Information Monte Carlo (PIMC).
    /// It samples possible hidden card distributions to handle incomplete information and plays out the game move-by-move. This is closest to a "real" AI player.
    Playout {
        /// Path to the JSON context file
        #[arg(short, long)]
        context: String,
    },
    /// Plays out the game from the given state using Perfect Information.
    /// It assumes all cards are known to all players (open hand) and executes the optimal line of play to determine the final score.
    StandardPlayout {
        /// Path to the JSON context file
        #[arg(short, long)]
        context: String,
    },
    /// Plays out the game with full analysis at every step.
    /// Like StandardPlayout, but at EACH move it calculates and prints the value of ALL possible legal moves. Useful for understanding "why" a move was made.
    AnalysisPlayout {
        /// Path to the JSON context file
        #[arg(short, long)]
        context: String,
    },
    /// Performs a single-step analysis of the provided game state.
    /// It calculates the value (score or win/loss) of every legal move available to the current player. Useful for evaluating a single decision point.
    Analysis {
        /// Path to the JSON context file
        #[arg(short, long)]
        context: String,
    },
    /// Evaluates the best Skat discard for a 12-card hand.
    /// It iterates through all possible 2-card discards and solves the resulting 10-card game to find the discard that maximizes the game value.
    /// Modes:
    /// - "best": Finds the single best discard pair (fastest).
    /// - "all": Lists all possible discards sorted by value.
    /// - "win": Checks which discards lead to a win (>= 61 points).
    SkatCalc {
        /// Path to the JSON context file (must contain 12 declarer cards)
        #[arg(short, long)]
        context: String,
        /// Calculation mode: "best", "all", or "win"
        #[arg(short, long, default_value = "best")]
        mode: String,
    },
    /// Determines the optimal game contract (Grand, Suit, Null) for a 12-card hand.
    /// It evaluates all valid game announcements and finds the one yielding the highest score or winning chance.
    /// Modes:
    /// - "best": Returns the game configuration that yields the highest point value.
    /// - "win": Returns the game configuration that guarantees a win (>= 61 points).
    BestGame {
        /// Path to the JSON context file (must contain 12 declarer cards)
        #[arg(short, long)]
        context: String,
        /// Calculation mode: "best" or "win"
        #[arg(short, long, default_value = "best")]
        mode: String,
    },
    /// Calculates the value of a game state with incomplete information using PIMC.
    /// PIMC (Perfect Information Monte Carlo) samples possible distributions of unknown cards.
    /// Modes:
    /// - "win": Estimates the probability of winning (Declarer >= 61 or Null logic).
    /// - "best": Estimates value of all possible moves for the current player.
    /// Calculates the value of a game state with incomplete information using PIMC.
    /// PIMC (Perfect Information Monte Carlo) samples possible distributions of unknown cards.
    ///
    /// ARGS:
    ///     --context <FILE>    Path to JSON context file (30 cards for auto-skat).
    ///     --mode <MODE>       "win" (default) or "best".
    ///                         - "win": Win probability for current state.
    ///                         - "best": Win probability for each legal move.
    ///     --log-file <PATH>   Optional. Writes sample details to this file.
    PimcCalc {
        /// Path to the JSON context file
        #[arg(short, long)]
        context: String,
        /// Calculation mode: "best" or "win"
        #[arg(short, long, default_value = "win")]
        mode: String,
        /// Optional path to a log file to write sample details to
        #[arg(long)]
        log_file: Option<String>,
    },
    /// Calculate best game for a given hand using PIMC.
    /// It evaluates all valid game announcements and finds the one yielding the highest score or winning chance,
    /// considering incomplete information through PIMC sampling.
    PimcBestGame {
        /// Path to the JSON context file (must contain 12 declarer cards)
        #[arg(short, long)]
        context: String,
        /// Number of PIMC samples to run for each game type evaluation
        #[arg(short, long, default_value_t = 20)]
        samples: u32,
        /// Optional path to a log file to write sample details to
        #[arg(long)]
        log_file: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameContextInput {
    pub declarer_cards: String,
    pub left_cards: String,
    pub right_cards: String,
    pub game_type: Game,
    pub start_player: Player,
    pub mode: Option<SearchMode>, // Made option to allow missing for Playout if needed, or re-use
    pub trick_cards: Option<String>,
    pub trick_suit: Option<String>,
    pub declarer_start_points: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SearchMode {
    Win,
    Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PimcFactsInput {
    pub no_trump: Option<bool>,
    pub no_clubs: Option<bool>,
    pub no_spades: Option<bool>,
    pub no_hearts: Option<bool>,
    pub no_diamonds: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PimcPlayerFactsInput {
    pub declarer: Option<PimcFactsInput>,
    pub left: Option<PimcFactsInput>,
    pub right: Option<PimcFactsInput>,
}

#[derive(Debug, Deserialize)]
pub struct PimcBestGameInput {
    pub my_cards: String,
    pub start_player: Player,
    pub description: Option<String>,
    pub usage: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PimcContextInput {
    pub game_type: Game,
    pub my_player: Player,
    pub my_cards: String,
    pub remaining_cards: String,
    pub trick_cards: Option<String>,
    pub trick_suit: Option<String>,
    // PIMC specific optional fields for mid-trick or detailed setup
    pub previous_card: Option<String>,
    pub next_card: Option<String>,
    pub declarer_start_points: Option<u8>,
    pub threshold: Option<u8>,
    pub samples: Option<u32>,
    pub facts: Option<PimcPlayerFactsInput>,
}
