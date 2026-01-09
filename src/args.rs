use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use skat_aug23::skat::defs::{Game, Player};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Calculate the value of a game
    ValueCalc {
        /// Path to the JSON context file
        #[arg(short, long)]
        context: String,
    },
    /// Play out a game using PIMC
    Playout {
        /// Path to the JSON context file
        #[arg(short, long)]
        context: String,
    },
    /// Play out a game using Standard Perfect Information
    StandardPlayout {
        /// Path to the JSON context file
        #[arg(short, long)]
        context: String,
    },
    /// Play out a game with full analysis of all moves at each step
    AnalysisPlayout {
        /// Path to the JSON context file
        #[arg(short, long)]
        context: String,
    },
    /// Analyze the single initial state (all allowed moves values)
    Analysis {
        /// Path to the JSON context file
        #[arg(short, long)]
        context: String,
    },
    /// Calculate the best skat for a declarer hand of 12 cards
    SkatCalc {
        /// Path to the JSON context file
        #[arg(short, long)]
        context: String,
        /// Calculation mode: "best", "all", or "win"
        #[arg(short, long, default_value = "best")]
        mode: String,
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
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SearchMode {
    Win,
    Value,
}
