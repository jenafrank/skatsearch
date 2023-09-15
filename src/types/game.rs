pub mod methods;
pub mod traits;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Game {
    Farbe,
    Grand,
    Null
}
