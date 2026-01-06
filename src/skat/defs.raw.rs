pub mod methods;
pub mod traits;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Game {
    Farbe,
    Grand,
    Null
}

impl Game {
    pub fn convert_to_string(&self) -> String {
        return match self {
            Game::Farbe => { "Farbe".to_string() }
            Game::Grand => { "Grand".to_string() }
            Game::Null => { "Null".to_string() }
        }
    }
}
pub mod methods;
pub mod traits;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Player {
    Declarer = 0,
    Left = 1,
    Right = 2,
}
   
impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {            
            Player::Declarer => write!(f, "Declarer"),
            Player::Left     => write!(f, "Left"),
            Player::Right    => write!(f, "Right")
        }
    }
}    
//! # bitboard.rs
//!
//! Contains important constants, mostly in binary format.
//! The specific notation points to a standard representation of
//! skat cards, i.e. 4 jacks, 7 clubs, 7 spades, 7 hearts and 7 diamonds.

// Trump cards

/// All trump cards for a suit game. The suit color is always clubs.
pub const TRUMP_FARBE: u32 = 0b1111_1111111_0000000_0000000_0000000;

/// All trump cards for a grand game. That is only the four jacks.
pub const TRUMP_GRAND: u32 = 0b1111_0000000_0000000_0000000_0000000;

/// All trump cards for a null game. Equals zero since there is no trump
/// in null games.
pub const TRUMP_NULL: u32 = 0u32;

/// Mask for all clubs cards except the club jack. Used in grand games.
pub const CLUBS: u32 = 0b0000_1111111_0000000_0000000_0000000;

/// Mask for all spades cards except the spades jack. Used in grand and suit games.
pub const SPADES: u32 = 0b0000_0000000_1111111_0000000_0000000;

/// Mask for all hearts cards except the hearts jack. Used in grand and suit games.
pub const HEARTS: u32 = 0b0000_0000000_0000000_1111111_0000000;

/// Mask for all diamond cards except the diamond jack. Used in grand and suit games.
pub const DIAMONDS: u32 = 0b0000_0000000_0000000_0000000_1111111;

/// Mask for all clubs cards. Used in null games.
pub const NULL_CLUBS: u32 = 0b1000_1111111_0000000_0000000_0000000;

/// Mask for all spades cards. Used in null games.
pub const NULL_SPADES: u32 = 0b0100_0000000_1111111_0000000_0000000;

/// Mask for all hearts cards. Used in null games.
pub const NULL_HEARTS: u32 = 0b0010_0000000_0000000_1111111_0000000;

/// Mask for all diamonds cards. Used in null games.
pub const NULL_DIAMONDS: u32 = 0b0001_0000000_0000000_0000000_1111111;

/// Mask for all jacks cards.
pub const JACKS : u32 = 0b_1111_0000000_0000000_0000000_0000000;

/// Mask for all aces cards.
pub const ACES  : u32 = 0b_0000_1000000_1000000_1000000_1000000;

/// Mask for all tens cards.
pub const TENS  : u32 = 0b_0000_0100000_0100000_0100000_0100000;

/// Mask for all kings cards.
pub const KINGS : u32 = 0b_0000_0010000_0010000_0010000_0010000;

/// Mask for all queens cards.
pub const QUEENS: u32 = 0b_0000_0001000_0001000_0001000_0001000;

/// Mask for all nines cards.
pub const NINES : u32 = 0b_0000_0000100_0000100_0000100_0000100;

/// Mask for all eights cards.
pub const EIGHTS: u32 = 0b_0000_0000010_0000010_0000010_0000010;

/// Mask for all sevens cards.
pub const SEVENS: u32 = 0b_0000_0000001_0000001_0000001_0000001;

/// Mask for all cards.
pub const ALLCARDS: u32 = 0b_1111_1111111_1111111_1111111_1111111;

// Masks for all single cards of the 32 skat cards.
pub const JACKOFCLUBS:     u32 = 0b_1000_0000000_0000000_0000000_0000000;
pub const JACKOFSPADES:    u32 = 0b_0100_0000000_0000000_0000000_0000000;
pub const JACKOFHEARTS:    u32 = 0b_0010_0000000_0000000_0000000_0000000;
pub const JACKOFDIAMONDS:  u32 = 0b_0001_0000000_0000000_0000000_0000000;

pub const ACEOFCLUBS:      u32 = 0b_0000_1000000_0000000_0000000_0000000;
pub const TENOFCLUBS:      u32 = 0b_0000_0100000_0000000_0000000_0000000;
pub const KINGOFCLUBS:     u32 = 0b_0000_0010000_0000000_0000000_0000000;
pub const QUEENOFCLUBS:    u32 = 0b_0000_0001000_0000000_0000000_0000000;
pub const NINEOFCLUBS:     u32 = 0b_0000_0000100_0000000_0000000_0000000;
pub const EIGHTOFCLUBS:    u32 = 0b_0000_0000010_0000000_0000000_0000000;
pub const SEVENOFCLUBS:    u32 = 0b_0000_0000001_0000000_0000000_0000000;

pub const ACEOFSPADES:     u32 = 0b_0000_0000000_1000000_0000000_0000000;
pub const TENOFSPADES:     u32 = 0b_0000_0000000_0100000_0000000_0000000;
pub const KINGOFSPADES:    u32 = 0b_0000_0000000_0010000_0000000_0000000;
pub const QUEENOFSPADES:   u32 = 0b_0000_0000000_0001000_0000000_0000000;
pub const NINEOFSPADES:    u32 = 0b_0000_0000000_0000100_0000000_0000000;
pub const EIGHTOFSPADES:   u32 = 0b_0000_0000000_0000010_0000000_0000000;
pub const SEVENOFSPADES:   u32 = 0b_0000_0000000_0000001_0000000_0000000;

pub const ACEOFHEARTS:     u32 = 0b_0000_0000000_0000000_1000000_0000000;
pub const TENOFHEARTS:     u32 = 0b_0000_0000000_0000000_0100000_0000000;
pub const KINGOFHEARTS:    u32 = 0b_0000_0000000_0000000_0010000_0000000;
pub const QUEENOFHEARTS:   u32 = 0b_0000_0000000_0000000_0001000_0000000;
pub const NINEOFHEARTS:    u32 = 0b_0000_0000000_0000000_0000100_0000000;
pub const EIGHTOFHEARTS:   u32 = 0b_0000_0000000_0000000_0000010_0000000;
pub const SEVENOFHEARTS:   u32 = 0b_0000_0000000_0000000_0000001_0000000;

pub const ACEOFDIAMONDS:   u32 = 0b_0000_0000000_0000000_0000000_1000000;
pub const TENOFDIAMONDS:   u32 = 0b_0000_0000000_0000000_0000000_0100000;
pub const KINGOFDIAMONDS:  u32 = 0b_0000_0000000_0000000_0000000_0010000;
pub const QUEENOFDIAMONDS: u32 = 0b_0000_0000000_0000000_0000000_0001000;
pub const NINEOFDIAMONDS:  u32 = 0b_0000_0000000_0000000_0000000_0000100;
pub const EIGHTOFDIAMONDS: u32 = 0b_0000_0000000_0000000_0000000_0000010;
pub const SEVENOFDIAMONDS: u32 = 0b_0000_0000000_0000000_0000000_0000001;


/// Indexes from 0 until 31 as array.
pub const RANGE: [usize; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
];

/// Indexes from 31 until 0 as array.
pub const RANGE_INV: [usize; 32] = [
    31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8,
    7, 6, 5, 4, 3, 2, 1, 0,
];

/// Short card name from index 0 until index 31.
pub const CARDS: [&str; 32] = [
    "CJ", "SJ", "HJ", "DJ", 
    "CA", "CT", "CK", "CQ", "C9", "C8", "C7",
    "SA", "ST", "SK", "SQ", "S9", "S8", "S7",
    "HA", "HT", "HK", "HQ", "H9", "H8", "H7",
    "DA", "DT", "DK", "DQ", "D9", "D8", "D7"
    ];

// Connection constants

/// The connection breaker symbolizes the end of a sequence of rank-equivalent cards.
pub const CONNECTION_BREAKER: u32 = 0u32;

/// The sequence of rank-equivalent cards in suit game, basically ending with suit.
pub const FARB_CONN: [(u32,u8); 36] = [
    (JACKOFCLUBS, 2),
    (JACKOFSPADES, 2),
    (JACKOFHEARTS, 2),
    (JACKOFDIAMONDS, 2),
    (ACEOFCLUBS, 11),
    (TENOFCLUBS, 10),
    (KINGOFCLUBS, 4),
    (QUEENOFCLUBS, 3),
    (NINEOFCLUBS, 0),
    (EIGHTOFCLUBS, 0),
    (SEVENOFCLUBS, 0),

    (CONNECTION_BREAKER,0),

    (ACEOFSPADES, 11),
    (TENOFSPADES, 10),
    (KINGOFSPADES, 4),
    (QUEENOFSPADES, 3),
    (NINEOFSPADES, 0),
    (EIGHTOFSPADES, 0),
    (SEVENOFSPADES, 0),

    (CONNECTION_BREAKER,0),

    (ACEOFHEARTS, 11),
    (TENOFHEARTS, 10),
    (KINGOFHEARTS, 4),
    (QUEENOFHEARTS, 3),
    (NINEOFHEARTS, 0),
    (EIGHTOFHEARTS, 0),
    (SEVENOFHEARTS, 0),

    (CONNECTION_BREAKER,0),

    (ACEOFDIAMONDS, 11),
    (TENOFDIAMONDS, 10),
    (KINGOFDIAMONDS, 4),
    (QUEENOFDIAMONDS, 3),
    (NINEOFDIAMONDS, 0),
    (EIGHTOFDIAMONDS, 0),
    (SEVENOFDIAMONDS, 0),

    (CONNECTION_BREAKER,0),
];

/// The sequence of rank-equivalent cards in grand game, basically ending with suit or trump.
/// Used in (unequal) move reduction.
pub const GRAND_CONN: [(u32,u8); 37] = [
    (JACKOFCLUBS, 2),
    (JACKOFSPADES, 2),
    (JACKOFHEARTS, 2),
    (JACKOFDIAMONDS, 2),

    (CONNECTION_BREAKER,0),

    (ACEOFCLUBS, 11),
    (TENOFCLUBS, 10),
    (KINGOFCLUBS, 4),
    (QUEENOFCLUBS, 3),
    (NINEOFCLUBS, 0),
    (EIGHTOFCLUBS, 0),
    (SEVENOFCLUBS, 0),

    (CONNECTION_BREAKER,0),

    (ACEOFSPADES, 11),
    (TENOFSPADES, 10),
    (KINGOFSPADES, 4),
    (QUEENOFSPADES, 3),
    (NINEOFSPADES, 0),
    (EIGHTOFSPADES, 0),
    (SEVENOFSPADES, 0),

    (CONNECTION_BREAKER,0),

    (ACEOFHEARTS, 11),
    (TENOFHEARTS, 10),
    (KINGOFHEARTS, 4),
    (QUEENOFHEARTS, 3),
    (NINEOFHEARTS, 0),
    (EIGHTOFHEARTS, 0),
    (SEVENOFHEARTS, 0),

    (CONNECTION_BREAKER,0),

    (ACEOFDIAMONDS, 11),
    (TENOFDIAMONDS, 10),
    (KINGOFDIAMONDS, 4),
    (QUEENOFDIAMONDS, 3),
    (NINEOFDIAMONDS, 0),
    (EIGHTOFDIAMONDS, 0),
    (SEVENOFDIAMONDS, 0),

    (CONNECTION_BREAKER,0),
];

/// The sequence of value-equivalent cards in suit game. E.g. the first four jacks are
/// rank- and value-equivalent. E.g. 9, 8 and 7 are rank- and value-equivalent.
pub const FARB_CONN_EQ: [u32; 21] = [
    JACKOFCLUBS,
    JACKOFSPADES,
    JACKOFHEARTS,
    JACKOFDIAMONDS,

    CONNECTION_BREAKER,

    NINEOFCLUBS,
    EIGHTOFCLUBS,
    SEVENOFCLUBS,

    CONNECTION_BREAKER,

    NINEOFSPADES,
    EIGHTOFSPADES,
    SEVENOFSPADES,

    CONNECTION_BREAKER,

    NINEOFHEARTS,
    EIGHTOFHEARTS,
    SEVENOFHEARTS,

    CONNECTION_BREAKER,

    NINEOFDIAMONDS,
    EIGHTOFDIAMONDS,
    SEVENOFDIAMONDS,

    CONNECTION_BREAKER
];

/// The sequence of value-equivalent cards in grand game.
pub const GRAND_CONN_EQ: [u32; 21] = FARB_CONN_EQ;

/// The sequence of value-equivalent cards in null game. Ends, when suit ends.
/// In a null game, all cards can be regarded as having the same value (e.g. 1) and
/// the goal for the declarer is to make a total value of 0 happen.
pub const NULL_CONN_EQ: [u32; 36] = [
    ACEOFCLUBS,
    KINGOFCLUBS,
    QUEENOFCLUBS,
    JACKOFCLUBS,
    TENOFCLUBS,
    NINEOFCLUBS,
    EIGHTOFCLUBS,
    SEVENOFCLUBS,

    CONNECTION_BREAKER,

    ACEOFSPADES,
    KINGOFSPADES,
    QUEENOFSPADES,
    JACKOFSPADES,
    TENOFSPADES,
    NINEOFSPADES,
    EIGHTOFSPADES,
    SEVENOFSPADES,

    CONNECTION_BREAKER,

    ACEOFHEARTS,
    KINGOFHEARTS,
    QUEENOFHEARTS,
    JACKOFHEARTS,
    TENOFHEARTS,
    NINEOFHEARTS,
    EIGHTOFHEARTS,
    SEVENOFHEARTS,

    CONNECTION_BREAKER,

    ACEOFDIAMONDS,
    KINGOFDIAMONDS,
    QUEENOFDIAMONDS,
    JACKOFDIAMONDS,
    TENOFDIAMONDS,
    NINEOFDIAMONDS,
    EIGHTOFDIAMONDS,
    SEVENOFDIAMONDS,

    CONNECTION_BREAKER,
];
