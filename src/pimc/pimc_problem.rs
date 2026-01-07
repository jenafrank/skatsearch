use crate::skat::builder::GameContextBuilder;
use crate::skat::context::GameContext;
use crate::skat::defs::Game;
use crate::skat::defs::Player;
use crate::traits::{BitConverter, StringConverter};

use super::facts::Facts;

#[derive(Clone, Copy)]
pub struct PimcProblem {
    game_type: Game,
    my_player: Player,
    my_cards: u32,

    // Primary values
    previous_card: u32,
    next_card: u32,
    all_cards: u32,
    threshold: u8,

    // Facts
    facts_previous_player: Facts,
    facts_next_player: Facts,
}

impl PimcProblem {
    pub fn advance(&self, played_card: String) -> PimcProblem {
        let mut ret = self.clone();

        ret.print_object();

        // Check if facts are added
        // todo!() assuming no for now

        // Check if trick completed => Remove trick from table and adjust points
        // todo!() assuming no for now

        // Add card to trick card
        ret.previous_card = played_card.__bit();

        ret.print_object();

        ret
    }
    fn print_object(&self) {
        println!("");
        println!("DUMPING PIMC PROBLEM: ");
        println!("-------------------------- ");
        println!("game type = {}", self.game_type.convert_to_string());
        println!("my player = {}", self.my_player.str());
        println!("my cards = {}", self.my_cards.__str());
        println!(
            "trick_card_previous_player = {}",
            self.previous_card.__str()
        );
        println!("trick_card_next_player = {}", self.next_card.__str());
        println!("all cards = {}", self.all_cards.__str());
        println!("threshold = {}", self.threshold);
        println!(
            "facts left = {}",
            self.facts_previous_player.convert_to_string()
        );
        println!(
            "facts right = {}",
            self.facts_next_player.convert_to_string()
        );
    }
}

// Gettter
impl PimcProblem {
    pub fn game_type(&self) -> Game {
        self.game_type
    }

    pub fn my_player(&self) -> Player {
        self.my_player
    }

    pub fn my_cards(&self) -> u32 {
        self.my_cards
    }

    pub fn previous_card(&self) -> u32 {
        self.previous_card
    }

    pub fn next_card(&self) -> u32 {
        self.next_card
    }

    pub fn cards_on_table(&self) -> u32 {
        self.previous_card | self.next_card
    }

    pub fn all_cards(&self) -> u32 {
        self.all_cards
    }

    pub fn threshold(&self) -> u8 {
        self.threshold
    }

    pub fn facts_previous_player(&self) -> Facts {
        self.facts_previous_player
    }

    pub fn facts_next_player(&self) -> Facts {
        self.facts_next_player
    }
}

// Setter
impl PimcProblem {
    pub fn set_game_type(&mut self, game_type: Game) {
        self.game_type = game_type;
    }

    pub fn set_my_player(&mut self, my_player: Player) {
        self.my_player = my_player;
    }

    pub fn set_my_cards(&mut self, my_cards: u32) {
        self.my_cards = my_cards;
    }

    pub fn set_previous_card(&mut self, previous_card: u32) {
        self.previous_card = previous_card;
    }

    pub fn set_next_card(&mut self, next_card: u32) {
        self.next_card = next_card;
    }

    pub fn set_all_cards(&mut self, all_cards: u32) {
        self.all_cards = all_cards;
    }

    pub fn set_threshold(&mut self, threshold: u8) {
        self.threshold = threshold;
    }

    pub fn set_facts_left(&mut self, facts_left: Facts) {
        self.facts_previous_player = facts_left;
    }

    pub fn set_facts_right(&mut self, facts_right: Facts) {
        self.facts_next_player = facts_right;
    }
}

impl PimcProblem {
    pub fn new() -> Self {
        PimcProblem {
            game_type: Game::Suit,
            my_cards: 0u32,
            my_player: Player::Declarer,
            all_cards: 0u32,
            previous_card: 0u32,
            next_card: 0u32,
            threshold: 1u8,
            facts_previous_player: Facts::zero_fact(),
            facts_next_player: Facts::zero_fact(),
        }
    }

    pub fn generate_concrete_problem(&self) -> GameContext {
        self.validate();

        let problem = GameContextBuilder::new(self.game_type)
            .cards(Player::Declarer, "")
            .cards(Player::Left, "")
            .cards(Player::Right, "")
            .turn(self.my_player)
            .trick_from_uproblem(self.previous_card, self.next_card)
            .threshold(self.threshold)
            .set_cards_for_problem(self.my_cards, self.my_player)
            .set_cards_for_other_players(
                self.all_cards,
                self.previous_card,
                self.next_card,
                self.my_cards,
                self.my_player,
                self.next_player_facts(),
                self.previous_player_facts(),
            )
            .build();

        if verify_card_distribution(&problem) {
            return problem;
        } else {
            panic!("Something went wrong in randomly select cards with given facts.");
        }
    }

    fn next_player_facts(&self) -> Facts {
        self.facts_next_player
    }

    fn previous_player_facts(&self) -> Facts {
        self.facts_previous_player
    }

    fn validate(&self) {
        self.validate_all_cards();
    }

    fn validate_all_cards(&self) {
        assert!(self.all_cards & self.my_cards == self.my_cards);
        assert!(self.all_cards & self.cards_on_table() == self.cards_on_table());

        // currently uncertain problems can only be solved before a trick starts:
        assert!(self.all_cards.count_ones() % 3 == 0);
    }
}

fn verify_card_distribution(problem: &GameContext) -> bool {
    assert!(problem.declarer_cards() & problem.left_cards() == 0);
    assert!(problem.declarer_cards() & problem.right_cards() == 0);
    assert!(problem.left_cards() & problem.right_cards() == 0);

    return true;
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::PimcProblem;
    use crate::{
        pimc::pimc_problem::Facts,
        skat::defs::{Game, Player},
        traits::{BitConverter, StringConverter},
    };

    #[test]
    fn test_problem_generation() {
        let uproblem = PimcProblem {
            game_type: Game::Suit,
            all_cards: "CA CT SA ST HA HT DA DT D9".__bit(),
            my_cards: "CA CT SA".__bit(),
            next_card: 0u32,
            previous_card: 0u32,
            my_player: Player::Declarer,
            threshold: 1,
            facts_previous_player: Facts::one_fact(true, false, false, false, false),
            facts_next_player: Facts::zero_fact(),
        };

        let problem = uproblem.generate_concrete_problem();

        println!("Declarer cards: {}", problem.declarer_cards().__str());
        println!("Left cards    : {}", problem.left_cards().__str());
        println!("Right cards   : {}", problem.right_cards().__str());
    }

    #[test]
    fn test_inter_trick_problem_generation() {
        let uproblem = PimcProblem {
            game_type: Game::Suit,
            all_cards: "CA CT SA ST HA HT DA DT D9".__bit(),
            my_cards: "CA CT SA".__bit(),
            my_player: Player::Declarer,
            threshold: 1,
            previous_card: "ST".__bit(),
            next_card: 0u32,
            facts_previous_player: Facts::zero_fact(),
            facts_next_player: Facts::zero_fact(),
        };

        let problem = uproblem.generate_concrete_problem();

        println!("Declarer cards: {}", problem.declarer_cards().__str());
        println!("Left cards    : {}", problem.left_cards().__str());
        println!("Right cards   : {}", problem.right_cards().__str());
    }

    #[test]
    fn test_farbe_no_trump_fact() {
        // Game is Farbe (Clubs Trump).
        // Fact: Left player has NO Trump.
        // Expectation: Left player should NOT have Jacks or Clubs (TRUMP_SUIT).
        use crate::skat::defs::{DIAMONDS, HEARTS, SPADES, TRUMP_SUIT};

        let uproblem = PimcProblem {
            game_type: Game::Suit,
            // Total 6 cards. My=2. Left=2. Right=2.
            // My: SA ST (Non-Trump)
            // Available:
            // 2 Trump: CA CT (Clubs)
            // 2 Trump: HA HT (Wait! Hearts are NOT Trump in Clubs unless Jack. These are regular cards).
            // Actually, in `test_hearts_no_trump_fact`, HA HT were Trump because it was Hearts game.
            // Here it is Clubs game. So HA HT are valid non-trump.
            // But we want to test that Left (No Trump) avoids Trump.
            // So we need Trump cards available (CA, CT, CJ, SJ).
            // Let's use CA CT.
            // And some Non-Trump (HA, HT).
            // Left has No Trump -> Must take HA HT.
            all_cards: "SA ST CA CT HA HT".__bit(),
            my_cards: "SA ST".__bit(),
            my_player: Player::Declarer,
            threshold: 1,
            next_card: 0u32,
            previous_card: 0u32,
            facts_previous_player: Facts::zero_fact(),
            facts_next_player: Facts::one_fact(true, false, false, false, false), // No Trump for Left
        };

        // Run multiple times to ensure randomness doesn't accidentally succeed
        for _ in 0..20 {
            let problem = uproblem.generate_concrete_problem();
            let left_cards = problem.left_cards();

            // Left should have NO Trump cards (No Jacks, No Clubs)
            assert_eq!(
                left_cards & TRUMP_SUIT,
                0,
                "Left player has trump cards despite No Trump fact! Cards: {}",
                left_cards.__str()
            );

            // Left SHOULD have valid Non-Trump cards (Hearts in this case)
            assert_eq!(
                left_cards & (SPADES | DIAMONDS | HEARTS),
                left_cards,
                "Left should only have Non-Trump suits."
            );
        }
    }

    #[test]
    fn test_farbe_no_clubs_fact() {
        // Game is Farbe (Clubs Trump).
        // Fact: Left player has NO Clubs.
        // Expectation: Left player should NOT have Cards of the Club Suit (7-A).
        // Since Clubs is Trump, this largely overlaps with No Trump, but technically "No Clubs" fact specifically targets Club suit.
        // In our engine, No Clubs (fact) in Farbe -> Remove TRUMP_SUIT (Jacks + Clubs).
        // So they should have no Clubs AND no Jacks.

        use crate::skat::defs::TRUMP_SUIT;

        let uproblem = PimcProblem {
            game_type: Game::Suit,
            // Total 6 cards. My=2. Left=2. Right=2.
            // My: HA HT (Hearts - Non-Trump)
            // Available:
            // 2 Clubs: CA CT (Trump)
            // 2 Spades: SA ST (Non-Trump)
            // Left has No Clubs. So can't take CA CT.
            // Must take SA ST.
            all_cards: "HA HT CA CT SA ST".__bit(),
            my_cards: "HA HT".__bit(),
            my_player: Player::Declarer,
            threshold: 1,
            next_card: 0u32,
            previous_card: 0u32,
            facts_previous_player: Facts::zero_fact(),
            facts_next_player: Facts::one_fact(false, true, false, false, false), // No Clubs for Left
        };

        for _ in 0..20 {
            let problem = uproblem.generate_concrete_problem();
            let left_cards = problem.left_cards();

            // Left should have NO Club suit cards (and no Jacks because logic maps No Clubs -> No Trump in Farbe)
            assert_eq!(
                left_cards & TRUMP_SUIT,
                0,
                "Left player has Trump cards despite No Clubs fact!"
            );
        }
    }

    #[test]
    fn test_farbe_no_spades_fact() {
        // Game is Farbe (Clubs Trump).
        // Fact: Left player has NO Spades.
        // Expectation: Left player should have no SPADES suit cards.
        // BUT they CAN have Jack of Spades (JS), because JS is Trump, not Spades.
        // The implementation matches `ret_cards & !SPADES`. `SPADES` constant excludes JS.

        use crate::skat::defs::SPADES;

        let uproblem = PimcProblem {
            game_type: Game::Suit,
            // Total 6 cards. My=2. Left=2. Right=2.
            // My: CA CT (Clubs - Trump)
            // Available:
            // 2 Spades: SA ST
            // 1 Spade Jack: SJ (Trump)
            // 1 Heart: HA
            // Left has No Spades -> Can't take SA ST.
            // Available non-Spades: SJ, HA.
            // So Left MUST take SJ and HA.
            // (SJ is technically a Spade card physically, but logic-wise it's Trump. Does 'No Spades' fact exclude it?)
            // Usually 'No Spades' means 'Cannot follow Spades lead'. A Spades lead requires Spades suit. JS is Trump.
            // So 'No Spades' allows JS.
            all_cards: "CA CT SA ST SJ HA".__bit(),
            my_cards: "CA CT".__bit(),
            my_player: Player::Declarer,
            threshold: 1,
            next_card: 0u32,
            previous_card: 0u32,
            facts_previous_player: Facts::zero_fact(),
            facts_next_player: Facts::one_fact(false, false, true, false, false), // No Spades for Left
        };

        for _ in 0..20 {
            let problem = uproblem.generate_concrete_problem();
            let left_cards = problem.left_cards();

            // Left should have NO Spades suit cards (defined by SPADES constant)
            assert_eq!(
                left_cards & SPADES,
                0,
                "Left player has Spades suit cards despite No Spades fact!"
            );

            // Left SHOULD have SJ (which is 0x0100 in Spades suit technically, but mapped to Jack?)
            // Wait, card bits are unique. `SPADES` mask usually excludes the Jack bit.
            // Verify this with the test.
            // Left SHOULD have SJ.
            // We can check if SJ is in left_cards by bitwise AND.
            // Need the bit for SJ.
            let sj_bit = "SJ".__bit();
            assert!(
                left_cards & sj_bit != 0,
                "Left player should be allowed to have SJ despite No Spades fact."
            );
        }
    }

    #[test]
    fn test_grand_no_trump_fact() {
        // Grand Game. Trump is ONLY Jacks.
        // Fact: No Trump.
        // Left should have no Jacks.
        use crate::skat::defs::JACKS;

        let uproblem = PimcProblem {
            game_type: Game::Grand,
            // Total 6 cards. My=2. Left=2. Right=2.
            // My: CA CT (Non-Trump)
            // Available:
            // 2 Jacks: CJ SJ
            // 2 Non-Jacks: SA ST
            // Left has No Trump -> Must take SA ST.
            all_cards: "CA CT CJ SJ SA ST".__bit(),
            my_cards: "CA CT".__bit(),
            my_player: Player::Declarer,
            threshold: 1,
            next_card: 0u32,
            previous_card: 0u32,
            facts_previous_player: Facts::zero_fact(),
            facts_next_player: Facts::one_fact(true, false, false, false, false), // No Trump for Left
        };

        for _ in 0..20 {
            let problem = uproblem.generate_concrete_problem();
            let left_cards = problem.left_cards();

            assert_eq!(
                left_cards & JACKS,
                0,
                "Left player has Jacks despite No Trump fact in Grand!"
            );
        }
    }

    #[test]
    fn test_null_no_clubs_fact() {
        // Game is Null. No Trump. Jacks are part of suits.
        // Fact: Left player has NO Clubs.
        // Expectation: Left player should NOT have any Club cards (including Jack of Clubs).
        use crate::skat::defs::NULL_CLUBS;

        let uproblem = PimcProblem {
            game_type: Game::Null,
            // Total 6 cards. My=2. Left=2. Right=2.
            // My: HA HT (Hearts)
            // Available:
            // 2 Clubs: CA, CJ (Jack is part of Club suit in Null)
            // 2 Spades: SA, ST
            // Left has No Clubs -> Must take SA ST.
            all_cards: "HA HT CA CJ SA ST".__bit(),
            my_cards: "HA HT".__bit(),
            my_player: Player::Declarer,
            threshold: 1,
            next_card: 0u32,
            previous_card: 0u32,
            facts_previous_player: Facts::zero_fact(),
            facts_next_player: Facts::one_fact(false, true, false, false, false), // No Clubs for Left
        };

        for _ in 0..20 {
            let problem = uproblem.generate_concrete_problem();
            let left_cards = problem.left_cards();

            assert_eq!(
                left_cards & NULL_CLUBS,
                0,
                "Left player has Clubs (or CJ) despite No Clubs fact in Null game! Cards: {}",
                left_cards.__str()
            );
        }
    }

    #[test]
    fn test_null_no_spades_fact() {
        // Game is Null.
        // Fact: Left player has NO Spades.
        // Expectation: Left player should NOT have any Spades (including Jack of Spades).
        use crate::skat::defs::NULL_SPADES;

        let uproblem = PimcProblem {
            game_type: Game::Null,
            // Total 6 cards.
            // My: HA HT
            // Available:
            // 2 Spades: SA, SJ (Jack is Spade)
            // 2 Diamonds: DA, DT
            // Left has No Spades -> Must take DA DT.
            all_cards: "HA HT SA SJ DA DT".__bit(),
            my_cards: "HA HT".__bit(),
            my_player: Player::Declarer,
            threshold: 1,
            next_card: 0u32,
            previous_card: 0u32,
            facts_previous_player: Facts::zero_fact(),
            facts_next_player: Facts::one_fact(false, false, true, false, false), // No Spades for Left
        };

        for _ in 0..20 {
            let problem = uproblem.generate_concrete_problem();
            let left_cards = problem.left_cards();

            assert_eq!(
                left_cards & NULL_SPADES,
                0,
                "Left player has Spades (or SJ) despite No Spades fact in Null game! Cards: {}",
                left_cards.__str()
            );
        }
    }

    #[test]
    fn test_null_no_hearts_fact() {
        // Game is Null.
        // Fact: Left player has NO Hearts.
        use crate::skat::defs::NULL_HEARTS;

        let uproblem = PimcProblem {
            game_type: Game::Null,
            // Total 6 cards.
            // My: CA CT
            // Available:
            // 2 Hearts: HA, HJ
            // 2 Diamonds: DA, DT
            // Left has No Hearts -> Must take DA DT.
            all_cards: "CA CT HA HJ DA DT".__bit(),
            my_cards: "CA CT".__bit(),
            my_player: Player::Declarer,
            threshold: 1,
            next_card: 0u32,
            previous_card: 0u32,
            facts_previous_player: Facts::zero_fact(),
            facts_next_player: Facts::one_fact(false, false, false, true, false), // No Hearts for Left
        };

        for _ in 0..20 {
            let problem = uproblem.generate_concrete_problem();
            let left_cards = problem.left_cards();

            assert_eq!(
                left_cards & NULL_HEARTS,
                0,
                "Left player has Hearts (or HJ) despite No Hearts fact in Null game! Cards: {}",
                left_cards.__str()
            );
        }
    }

    #[test]
    fn test_null_no_diamonds_fact() {
        // Game is Null.
        // Fact: Left player has NO Diamonds.
        use crate::skat::defs::NULL_DIAMONDS;

        let uproblem = PimcProblem {
            game_type: Game::Null,
            // Total 6 cards.
            // My: CA CT
            // Available:
            // 2 Diamonds: DA, DJ
            // 2 Spades: SA, ST
            // Left has No Diamonds -> Must take SA ST.
            all_cards: "CA CT DA DJ SA ST".__bit(),
            my_cards: "CA CT".__bit(),
            my_player: Player::Declarer,
            threshold: 1,
            next_card: 0u32,
            previous_card: 0u32,
            facts_previous_player: Facts::zero_fact(),
            facts_next_player: Facts::one_fact(false, false, false, false, true), // No Diamonds for Left
        };

        for _ in 0..20 {
            let problem = uproblem.generate_concrete_problem();
            let left_cards = problem.left_cards();

            assert_eq!(
                left_cards & NULL_DIAMONDS,
                0,
                "Left player has Diamonds (or DJ) despite No Diamonds fact in Null game! Cards: {}",
                left_cards.__str()
            );
        }
    }
}
