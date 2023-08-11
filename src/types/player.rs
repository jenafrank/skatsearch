pub mod methods;
pub mod traits;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Player {
    Declarer = 0,
    Left = 1,
    Right = 2,
}