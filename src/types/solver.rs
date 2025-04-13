use super::{problem::Problem, tt_table::TtTable};

pub mod solve;
pub mod playout_row;
pub mod playout;
pub mod get;
pub mod retargs;

pub struct Solver {
    pub problem: Problem,
    pub tt: TtTable
}
impl Solver {
    pub fn new(concrete_problem: Problem, tt: Option<TtTable>) -> Solver {
        Solver {
            problem: concrete_problem,
            tt: tt.unwrap_or_else(TtTable::new), // Verwende die gegebene TtTable oder erstelle eine neue
        }
    }
}
