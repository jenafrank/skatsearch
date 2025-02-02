use crate::types::{problem::Problem, tt_table::TtTable};
use super::Solver;

impl Solver {
    pub fn create(problem: Problem) -> Solver {
        Solver {
            problem
        }
    }

    pub fn create_with_new_transposition_table(problem: Problem) -> Solver {
        TtTable::reset();
        Solver::create(problem)
    }
}