use super::*;

impl Player {
    pub fn inc(&self) -> Player {
        match self {
            Player::Declarer => Player::Left,
            Player::Left => Player::Right,
            Player::Right => Player::Declarer
        }
    }

    pub fn dec(&self) -> Player {
        self.inc().inc()
    }

    pub fn str(&self) -> &str {
        match self {
            Player::Declarer => "D",
            Player::Left => "L",
            Player::Right => "R"
        }
    }

    pub fn is_team(&self) -> bool {
        !self.is_declarer()
    }

    pub fn is_declarer(&self) -> bool {
        matches!(self, Player::Declarer)
    }

    pub fn is_same_team_as(&self, player: Player) -> bool {
        self.is_team() == player.is_team()
    }
}
