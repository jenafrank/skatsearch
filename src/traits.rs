pub mod bit_converter;
pub mod bitboard;
pub mod points;
pub mod string_converter;

pub trait BitConverter {
    fn __bit(&self) -> u32;
}

pub trait Points {
    fn points(&self) -> u8;
    fn trick_points(&self) -> u8;
    fn card_count(&self) -> u8;
    fn card_points(&self) -> u8;
}

pub trait Bitboard {
    fn __contain(&self, card: u32) -> bool;
    fn __is_odd(&self) -> bool;
    fn __decompose(&self) -> ([u32; 32], usize);
    fn __decompose_twelve(&self) -> [u32; 12];
}

pub trait StringConverter {
    fn __str(&self) -> String;
}
