pub mod methods;
pub mod traits;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Game {
    Farbe,
    Grand,
    Null
}

impl Game {
    pub(crate) fn convert_to_string(&self) -> String {
        return match self {
            Game::Farbe => { "Farbe".to_string() }
            Game::Grand => { "Grand".to_string() }
            Game::Null => { "Null".to_string() }
        }
    }
}
