use crate::types::problem::Problem;
use super::Solver;

impl Solver {
    pub fn create(problem: Problem) -> Solver {
        Solver {
            problem
        }
    }

    pub fn create_with_new_transposition_table(problem: Problem) -> Solver {
        // If reset, is much slower, tested with 1M sized TT:
        // TtTable::invalidate();
        Solver::create(problem)
    }
}
