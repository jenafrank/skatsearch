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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameContextInput {
    pub declarer_cards: String,
    pub left_cards: String,
    pub right_cards: String,
    pub game_type: Game,
    pub start_player: Player,
    pub mode: SearchMode,
    pub trick_cards: Option<String>,
    pub trick_suit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SearchMode {
    Win,
    Value,
}
