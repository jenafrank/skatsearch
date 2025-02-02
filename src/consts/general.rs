//! # general.rs
//!
//! General constants used in core functions. Does not cover bitboard notation.
//! See bitboard.rs for that.

/// Value used in algorithm for calculating fast FNV hash.
pub const HSH_INIT: u64 = 0xcbf29ce484222325u64;

/// Value used in algorithm for calculating fast FNV hash.
pub const HSH_MUL: u64 = 0x00000100000001B3;

/// Size of transposition table as usize.
pub const TT_SIZE: usize = 1024*1024usize;

/// Size of transposition table transformed to u64 type.
pub const TT_SIZE_U64: u64 = TT_SIZE as u64;

/// Value of cards in bitboard notation from left to right. Standard order.
/// Maps to indices 0..31
pub const AUGENLIST: [u8; 32] = [
    2, 2, 2, 2, 11, 10, 4, 3, 0, 0, 0, 11, 10, 4, 3, 0, 0, 0, 11, 10, 4, 3, 0, 0, 0, 11, 10, 4, 3, 0,
    0, 0,
];

/// Value of cards in bitboard notation from right to left. Inverse order.
/// Maps to indices 31..0
pub const REVERSED_AUGENLIST: [u8; 32] = [
    0, 0, 0, 3, 4, 10, 11, 0, 0, 0, 3, 4, 10, 11, 0, 0, 0, 3, 4, 10, 11, 0, 0, 0, 3, 4, 10, 11, 2, 2,
    2, 2,
];

/// Special sort value for usage in move sorting.
pub const SORT_AUGENLIST: [u8; 32] = [
    16, 15, 14, 13, 11, 10, 4, 3, 0, 0, 0, 11, 10, 4, 3, 0, 0, 0, 11, 10, 4, 3, 0, 0, 0, 11, 10, 4, 3, 0,
    0, 0,
];

