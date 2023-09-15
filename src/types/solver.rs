use super::problem::Problem;

pub mod solve;
pub mod constructors;
pub mod playout_row;
pub mod playout;
pub mod get;
pub mod retargs;

pub struct Solver {
    pub problem: Problem
}
