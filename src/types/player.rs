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
