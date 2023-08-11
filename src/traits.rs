pub mod bit_converter;
pub mod augen;
pub mod string_converter;
pub mod bitboard;

pub trait BitConverter {
    fn __bit(&self) -> u32;
}

pub trait Augen {
    fn __get_value(&self) -> u8;
    fn __get_value_of_three_cards(&self) -> u8;
    fn __get_number_of_bits(&self) -> u8;
    fn __get_from_one_card(&self) -> u8;
}

pub trait Bitboard {
    fn __contain(&self, card: u32) -> bool;
    fn __is_odd(&self) -> bool;
    fn __decompose(&self) -> ([u32; 10],usize);
    fn __decompose_twelve(&self) -> [u32; 12];

}

pub trait StringConverter {
    fn __str(&self) -> String;
}
