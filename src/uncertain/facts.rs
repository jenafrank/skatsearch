#[derive(Clone, Copy)]
pub struct Facts {
    pub no_trump: bool,
    pub no_clubs: bool,
    pub no_spades: bool,
    pub no_hearts: bool,
    pub no_diamonds: bool,
}

impl Facts {
    pub fn convert_to_string(&self) -> String {
        let ret = format!("{} {} {} {} - {}", self.no_clubs, self.no_diamonds, self.no_hearts, self.no_spades, self.no_trump);
        ret
    }
}

impl Default for Facts {
    fn default() -> Self {
        Self { 
            no_trump: false, 
            no_clubs: false, 
            no_spades: false, 
            no_hearts: false, 
            no_diamonds: false }
    }
}

impl Facts {
    pub fn new() -> Self {
        Facts {
            no_trump: false,
            no_clubs: false,
            no_spades: false,
            no_hearts: false,
            no_diamonds: false,
        }
    }

    pub fn zero_fact() -> Facts {
        Facts{ no_trump: false, no_clubs: false, no_spades: false, no_hearts: false, no_diamonds: false }
    }

    pub fn one_fact(no_trump: bool, no_clubs: bool, no_spades: bool, no_hearts: bool, no_diamonds: bool) -> Facts {
        Facts{ no_trump: no_trump, no_clubs: no_clubs, no_spades: no_spades, no_hearts: no_hearts, no_diamonds: no_diamonds }
    }
}