
use crate::types::solver::Solver;

impl Solver {
    pub fn generate_skat_combinations(cards: &[u32]) -> Vec<(u32, u32)> {
        let mut combinations = Vec::new();
        for i in 0..11 {
            for j in i + 1..12 {
                combinations.push((cards[i], cards[j]));
            }
        }
        combinations
    }
}
