use crate::types::problem::Problem;
use super::Solver;

impl Solver {
    pub fn create(problem: Problem) -> Solver {
        Solver {
            problem
        }
    }
}