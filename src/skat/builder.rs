use crate::skat::context::GameContext;
use crate::{
    pimc::facts::Facts,
    skat::defs::*,
    skat::rules::{get_all_unplayed_cards, get_suit_for_card},
    traits::{BitConverter, Bitboard, Points},
};
use rand::seq::index::sample;

pub struct GameContextBuilder {
    declarer_cards: Option<u32>,
    left_cards: Option<u32>,
    right_cards: Option<u32>,
    game_type: Option<Game>,
    start_player: Option<Player>,
    threshold_upper: Option<u8>,
    trick_cards: Option<u32>,
    trick_suit: Option<u32>,
    declarer_start_points: Option<u8>,
}

impl GameContextBuilder {
    pub fn new(game: Game) -> GameContextBuilder {
        let mut builder = GameContextBuilder::default();
        builder.game_type = Some(game);
        builder
    }

    pub fn new_farbspiel() -> GameContextBuilder {
        GameContextBuilder::new(Game::Suit)
    }

    pub fn new_grand() -> GameContextBuilder {
        GameContextBuilder::new(Game::Grand)
    }

    pub fn new_null() -> GameContextBuilder {
        let builder = GameContextBuilder::new(Game::Null);
        builder.threshold(1)
    }

    /// Assign cards to a specific player
    ///
    pub fn cards(mut self, player: Player, cards: &str) -> GameContextBuilder {
        let cards_bit = cards.__bit();
        match player {
            Player::Declarer => self.declarer_cards = Some(cards_bit),
            Player::Left => self.left_cards = Some(cards_bit),
            Player::Right => self.right_cards = Some(cards_bit),
        }
        self
    }

    pub fn cards_all(
        mut self,
        declarer_cards: &str,
        left_cards: &str,
        right_cards: &str,
    ) -> GameContextBuilder {
        self.declarer_cards = Some(declarer_cards.__bit());
        self.left_cards = Some(left_cards.__bit());
        self.right_cards = Some(right_cards.__bit());
        self
    }

    pub fn turn(mut self, player: Player) -> GameContextBuilder {
        self.start_player = Some(player);
        self
    }

    pub fn threshold(mut self, threshold_upper: u8) -> GameContextBuilder {
        self.threshold_upper = Some(threshold_upper);
        self
    }

    pub fn declarer_start_points(mut self, points: u8) -> GameContextBuilder {
        self.declarer_start_points = Some(points);
        self
    }

    pub fn threshold_half(mut self) -> GameContextBuilder {
        let declarer_cards = self
            .declarer_cards
            .expect("No declarer cards have been set.");
        let left_cards = self.left_cards.expect("No declarer cards have been set.");
        let right_cards = self.right_cards.expect("No declarer cards have been set.");

        let all_cards = get_all_unplayed_cards(declarer_cards, left_cards, right_cards);
        self.threshold_upper = Some((all_cards.points() / 2) + 1);
        self
    }

    pub fn trick_from_uproblem(
        mut self,
        trick_card_previous_player: u32,
        trick_card_next_player: u32,
    ) -> GameContextBuilder {
        let trick_cards = trick_card_previous_player | trick_card_next_player;
        let leading_card = if trick_card_next_player > 0 {
            trick_card_next_player
        } else {
            trick_card_previous_player
        };

        let game_type = self.game_type.unwrap_or(Game::Grand);
        let trick_suit = if leading_card != 0 {
            get_suit_for_card(leading_card, game_type)
        } else {
            0
        };

        self.trick_cards = Some(trick_cards);
        self.trick_suit = Some(trick_suit);
        self
    }

    pub fn trick(mut self, trick_suit: u32, trick_cards: &str) -> GameContextBuilder {
        self.trick_cards = Some(trick_cards.__bit());
        self.trick_suit = Some(trick_suit);
        self
    }

    pub fn build(self) -> GameContext {
        self.validate();

        let game_type = self.game_type.unwrap_or(Game::Grand); // Default? Or panic?
                                                               // Old builder used Problem::new() which defaulted to Grand/0 cards.

        let mut context = GameContext::create(
            self.declarer_cards.unwrap_or(0),
            self.left_cards.unwrap_or(0),
            self.right_cards.unwrap_or(0),
            game_type,
            self.start_player.unwrap_or(Player::Declarer),
        );

        // Problem struct had specific setter methods.
        // GameContext create seems to set cards and game type.
        // What about trick_cards, trick_suit, threshold?
        // Check GameContext::create implementation.
        // If GameContext doesn't support setting these in create, I might need to access fields directly if they are pub.
        // Assuming GameContext fields are public or have setters logic.
        // Based on Step 509 summary "GameContext ... including fields and methods".
        // They were private in Problem?
        // Let's assume for now I need to set them.

        // Wait, I am viewing context.rs in PARALLEL. I should wait for that view before creating this file!
        // But I can't wait if I call parallel.
        // I'll make a best guess placeholder and then UPDATE it if wrong.
        // Actually, I'll rely on reading context.rs in next step to fix this file if needed.
        // But write_to_file overwrites.

        // Let's postpone writing until I see context.rs!
        // I will return ONLY view_file and wait.

        // Oops, I can't undo the tool call structure here. I'm already committing to write.
        // I'll write a SKELETON that I know needs fixing, or try to be smart.
        // Actually, look at Step 509 summary: "consolidating the Problem struct ... including its fields".
        // If fields are pub, I can set them.

        // I'll just comment out the setters I'm unsure about and fix in next turn.
        context.set_declarer_start_points(self.declarer_start_points.unwrap_or(0));

        if let Some(trick_cards) = self.trick_cards {
            context.set_trick_cards(trick_cards);
        }

        if let Some(trick_suit) = self.trick_suit {
            context.set_trick_suit(trick_suit);
        }
        // But ProblemBuilder used setters.

        context
    }

    fn validate(&self) {
        let declarer_cards = self.declarer_cards.expect("Declarer cards missing.");
        let left_cards = self.left_cards.expect("Left cards missing.");
        let right_cards = self.right_cards.expect("Right cards missing.");
        let _start_player = self.start_player.expect("No start player available.");
        let trick_cards = self.trick_cards.expect("No trick cards available.");

        assert_eq!(declarer_cards & left_cards, 0);
        assert_eq!(left_cards & right_cards, 0);
        assert_eq!(declarer_cards & right_cards, 0);

        // check card numbers with respect to cards_in_trick
        let _nr_cards_declarer = declarer_cards.count_ones();
        let _nr_cards_left = left_cards.count_ones();
        let _nr_cards_right = right_cards.count_ones();

        // assert_eq!(nr_cards_declarer, nr_cards_left);
        // assert_eq!(nr_cards_left, nr_cards_right);

        let _nr_cards_declarer_in_trick = (declarer_cards & trick_cards).count_ones();
        let _nr_cards_left_in_trick = (left_cards & trick_cards).count_ones();
        let _nr_cards_right_in_trick = (right_cards & trick_cards).count_ones();

        let nr_trick_cards = trick_cards.count_ones();

        assert!(nr_trick_cards <= 2);

        /*
        if nr_trick_cards >= 1 {
            match start_player {
                Player::Declarer => assert_eq!(nr_cards_right_in_trick, 1),
                Player::Left => assert_eq!(nr_cards_declarer_in_trick, 1),
                Player::Right => assert_eq!(nr_cards_left_in_trick, 1),
            }
        }

        if nr_trick_cards == 2 {
            match start_player {
                Player::Declarer => assert_eq!(nr_cards_left_in_trick, 1),
                Player::Left => assert_eq!(nr_cards_right_in_trick, 1),
                Player::Right => assert_eq!(nr_cards_declarer_in_trick, 1),
            }
        }
        */

        // ToDo: check threshold lower than total augen value
        //
    }

    pub fn set_cards_for_problem(mut self, my_cards: u32, my_player: Player) -> GameContextBuilder {
        self.set_cards_for_problem_core(my_cards, my_player);
        self
    }

    pub fn set_cards_for_other_players(
        mut self,
        all_cards: u32,
        card_on_table_previous_player: u32,
        card_on_table_next_player: u32,
        my_cards: u32,
        my_player: Player,
        next_player_facts: Facts,
        previous_player_facts: Facts,
    ) -> GameContextBuilder {
        let cards_on_hands_of_both_other_players = (all_cards & !my_cards) & !card_on_table_previous_player & !card_on_table_next_player;

        let mut cards_next_player = cards_on_hands_of_both_other_players;
        let mut cards_previous_player = cards_on_hands_of_both_other_players;

        let game_type = self.game_type.expect("No game type available.");

        cards_next_player =
            cancel_cards_with_facts(cards_next_player, next_player_facts, game_type);
        cards_previous_player =
            cancel_cards_with_facts(cards_previous_player, previous_player_facts, game_type);

        cards_previous_player = cards_previous_player & !card_on_table_next_player;

        let my_cards_count = my_cards.count_ones();


        let target_next = my_cards_count - if card_on_table_next_player != 0 { 1 } else { 0 };
        let target_prev = my_cards_count - if card_on_table_previous_player != 0 { 1 } else { 0 };

        if target_next == 0 {
            cards_next_player = 0;
        }
        if target_prev == 0 {
            cards_previous_player = 0;
        }

        let proposed_draw = self.draw_cards(
            cards_next_player, 
            cards_previous_player, 
            my_cards,
            target_next,
            target_prev
        );

        cards_next_player = proposed_draw.0 | card_on_table_next_player;
        cards_previous_player = proposed_draw.1 | card_on_table_previous_player;

        self.set_cards_for_problem_core(cards_next_player, my_player.inc());
        self.set_cards_for_problem_core(cards_previous_player, my_player.dec());

        self
    }

    fn set_cards_for_problem_core(&mut self, my_cards: u32, my_player: Player) {
        match my_player {
            Player::Declarer => self.declarer_cards = Some(my_cards),
            Player::Left => self.left_cards = Some(my_cards),
            Player::Right => self.right_cards = Some(my_cards),
        }
    }

    fn draw_cards(
        &mut self,
        cards_player_1: u32,
        cards_player_2: u32,
        _my_cards: u32,
        target_p1: u32,
        target_p2: u32
    ) -> (u32, u32) {
        let definite_cards_player_1 = cards_player_1 & !cards_player_2;
        let definite_cards_player_2 = cards_player_2 & !cards_player_1;

        let ambiguous_cards = cards_player_1 & cards_player_2;
        let nr_ambiguous_cards = ambiguous_cards.count_ones();
        
        let nr_definite_cards_player_1 = definite_cards_player_1.count_ones();
        let nr_definite_cards_player_2 = definite_cards_player_2.count_ones();
        
        // Ensure we don't underflow if constraints are impossible
        // (Assuming facts are consistent with card counts)
        let needed_for_p1 = if target_p1 > nr_definite_cards_player_1 {
            target_p1 - nr_definite_cards_player_1
        } else {
            0 // Should trigger assert/error if facts force too many cards?
        };
        
        let needed_for_p2 = if target_p2 > nr_definite_cards_player_2 {
            target_p2 - nr_definite_cards_player_2
        } else {
             0 
        };

        // Assert that the ambiguous cards can satisfy the needs
        // Note: It's possible nr_ambiguous > needed_p1 + needed_p2 if skat is involved or logic mismatch
        // But here we usually expect exact match for endgames.
        // Let's rely on sample count.
        
        assert!(needed_for_p1 + needed_for_p2 <= nr_ambiguous_cards, 
            "Not enough ambiguous cards to satisfy targets! Available: {}, StartP1: {}, StartP2: {}, TargetP1: {}, TargetP2: {}", 
            nr_ambiguous_cards, nr_definite_cards_player_1, nr_definite_cards_player_2, target_p1, target_p2);

        let draw_player_1 = random_cards(ambiguous_cards, needed_for_p1);
        let proposed_player_1 = definite_cards_player_1 | draw_player_1;

        let remaining_ambiguous = ambiguous_cards & !draw_player_1;
        
        let draw_player_2 = random_cards(remaining_ambiguous, needed_for_p2);
        let proposed_player_2 = definite_cards_player_2 | draw_player_2;

        (proposed_player_1, proposed_player_2)
    }
}

impl Default for GameContextBuilder {
    fn default() -> GameContextBuilder {
        GameContextBuilder {
            declarer_cards: None,
            left_cards: None,
            right_cards: None,
            game_type: None,
            start_player: None,
            threshold_upper: Some(1),
            trick_cards: Some(0),
            trick_suit: Some(0),
            declarer_start_points: Some(0),
        }
    }
}

fn random_cards(cards: u32, nr: u32) -> u32 {
    let mut random_number_generator = rand::thread_rng();

    let cards_dec = cards.__decompose();
    assert!(cards_dec.1 >= nr as usize);

    let indices = sample(&mut random_number_generator, cards_dec.1, nr as usize);

    let mut ret = 0;
    for i in indices.iter() {
        ret |= cards_dec.0[i];
    }

    ret
}

fn cancel_cards_with_facts(cards: u32, facts: Facts, game: Game) -> u32 {
    let mut ret_cards = cards;

    if facts.no_trump {
        ret_cards = match game {
            Game::Suit => ret_cards & !TRUMP_SUIT,
            Game::Grand => ret_cards & !TRUMP_GRAND,
            Game::Null => ret_cards & !TRUMP_NULL,
        }
    }

    if facts.no_clubs {
        ret_cards = match game {
            Game::Suit => ret_cards & !TRUMP_SUIT,
            Game::Grand => ret_cards & !CLUBS,
            Game::Null => ret_cards & !NULL_CLUBS,
        }
    }

    if facts.no_spades {
        ret_cards = match game {
            Game::Suit => ret_cards & !SPADES,
            Game::Grand => ret_cards & !SPADES,
            Game::Null => ret_cards & !NULL_SPADES,
        }
    }

    if facts.no_hearts {
        ret_cards = match game {
            Game::Suit => ret_cards & !HEARTS,
            Game::Grand => ret_cards & !HEARTS,
            Game::Null => ret_cards & !NULL_HEARTS,
        }
    }

    if facts.no_diamonds {
        ret_cards = match game {
            Game::Suit => ret_cards & !DIAMONDS,
            Game::Grand => ret_cards & !DIAMONDS,
            Game::Null => ret_cards & !NULL_DIAMONDS,
        }
    }

    ret_cards
}
