extern crate rand;

use rand::{thread_rng, random};
use rand::distributions::{Uniform, Distribution};

extern crate rusty_machine;
use rusty_machine::learning::nnet::{NeuralNet, BCECriterion};
use rusty_machine::learning::optim::grad_desc::StochasticGD;
use rusty_machine::linalg::Matrix;
use rusty_machine::learning::SupModel;

use std::fs::File;
use std::io::prelude::*;

const NUM_CARDS: usize = 40;
const NUM_PLAYERS: usize = 4;
const NUM_CARDS_PER_HAND: usize = 10;
const NUM_CARDS_PER_SUIT: usize = 10;

fn main() {
    let mut deck = generate_deck();
    let mut table = SuecaQLTable::new(0.05, 0.1);
    let mut total_score = (0u64, 0u64);
    let mut total_games = (0, 0);
    for i in 0..1000 {
        println!("Game {}", i);
        let score = game(&mut deck, &mut table);
        total_score.0 = total_score.0 + score.0 as u64;
        total_score.1 = total_score.1 + score.1 as u64;
        if score.0 > score.1 {
            total_games.0 = total_games.0 + 1;
        } else {
            total_games.1 = total_games.1 + 1;
        }
    }
    println!("{} - {}", total_score.0, total_score.1);
    println!("{} - {}", total_games.0, total_games.1);
    if let Ok(mut file) = File::create("result.txt") {
        let text = format!("{:?}", table);
        if file.write_all(text.as_bytes()).is_err() {
            println!("{}", text);
        }
    }
}

/*
    Card value:
    0 1 2 3 4 5 6 7 8 9
    2 3 4 5 6 Q J K 7 A
    
    Card suit:
    0 Hearts
    1 Diamonds
    2 Clubs
    3 Spades
*/

#[derive(Debug)]
struct SuecaQLTable {
    rnn: NeuralNet<'static, BCECriterion, StochasticGD>,
    alfa: f64,
    discount_factor: f64,
}


impl SuecaQLTable {
    fn new(alfa: f64, discount_factor: f64) -> SuecaQLTable {
        let layers = &[NUM_CARDS * 8, NUM_CARDS * 4, NUM_CARDS, 1];
        let rnn = NeuralNet::default(layers);
        SuecaQLTable {
            rnn,
            alfa,
            discount_factor,
        }
    }

    //TODO: update multiples in simultaneous

    fn update(&mut self, current_game_state: &GameState, game_states: [Option<GameState>; NUM_PLAYERS]) {
        let input_col_length = NUM_CARDS * 8;
        let mut input_data = Vec::with_capacity(input_col_length * NUM_PLAYERS);
        let mut target_data = Vec::with_capacity(NUM_PLAYERS);
        let winner_team = current_game_state.round.winner(current_game_state) % 2;
        let points = current_game_state.round.points();
        print!("\tReward: ");
        for (player, game_state) in game_states.iter().enumerate() {
            let game_state = match game_state {
                Some(game_state) => game_state,
                None => unreachable!()
            };
            input_data.append(&mut game_state.encode(player));
            let next_state = current_game_state.encode(player);
            let mut next_states = Vec::with_capacity(current_game_state.hands[player].len() * input_col_length);
            for card in current_game_state.hands[player].iter() {
                let mut next_state_action = {
                    let mut clone = next_state.clone();
                    clone[7 * NUM_CARDS + card.id as usize] = 1.0;
                    clone
                };
                next_states.append(&mut next_state_action);
            }
            let inputs = Matrix::new(current_game_state.hands[player].len(), input_col_length, next_states);
            let mut max_q = std::f64::NEG_INFINITY;
            for q_value in self.rnn.predict(&inputs).unwrap().data().iter() {
                if *q_value > max_q {
                    max_q = *q_value;
                }
            }
            let old_inputs = Matrix::new(1, input_col_length, current_game_state.encode(player));
            let old_q = self.rnn.predict(&old_inputs).unwrap().data()[0];
            let old = (1.0 - self.alfa) * old_q;
            let max_reward = 44.0;
            let reward = if player % 2 == winner_team {
                (points as f64 / max_reward) / 2.0 + 0.5
            } else {
                (-(points as f64) / max_reward) / 2.0 + 0.5
            };
            let new = if max_q.is_finite() {
                self.alfa * (reward + self.discount_factor * max_q)
            } else {
                self.alfa * reward
            };
            let q = old + new;
            print!("{:.3} ", new / self.alfa - old_q);
            target_data.push(q);
        }
        println!();
        let inputs = Matrix::new(NUM_PLAYERS, NUM_CARDS * 8, input_data);
        let targets = Matrix::new(NUM_PLAYERS, 1, target_data);
        if let Err(msg) = self.rnn.train(&inputs, &targets) {
            panic!(msg);
        }
    }

    // same as update but doesn't take into account next matches
    fn eager_update(&mut self, current_game_state: &GameState, game_states: [Option<GameState>; NUM_PLAYERS]) {
        let input_col_length = NUM_CARDS * 8;
        let mut input_data = Vec::with_capacity(input_col_length * NUM_PLAYERS);
        let mut target_data = Vec::with_capacity(NUM_PLAYERS);
        let winner_team = current_game_state.round.winner(current_game_state) % 2;
        let points = current_game_state.round.points();
        for (player, game_state) in game_states.iter().enumerate() {
            let game_state = match game_state {
                Some(game_state) => game_state,
                None => unreachable!()
            };
            input_data.append(&mut game_state.encode(player));
            let max_reward = 44.0;
            let reward = if player % 2 == winner_team {
                (points as f64 / max_reward) / 2.0 + 0.5
            } else {
                (-(points as f64) / max_reward) / 2.0 + 0.5
            };
            target_data.push(reward);
        }
        let inputs = Matrix::new(NUM_PLAYERS, NUM_CARDS * 8, input_data);
        let targets = Matrix::new(NUM_PLAYERS, 1, target_data);
        if let Err(msg) = self.rnn.train(&inputs, &targets) {
            panic!(msg);
        }
    }

    fn value(&self,
             player: usize,
             game_state: &GameState,
             card: &Card) -> f64 {
        let game_state = {
            let mut clone = game_state.clone();
            clone.round.played_cards[player] = Some(card.clone());
            clone
        };
        let input_data = game_state.encode(player);
        let inputs = Matrix::new(1, NUM_CARDS * 8, input_data);
        self.rnn.predict(&inputs).unwrap().data()[0]
    }

    fn choose_card(&self,
                   player: usize,
                   game_state: &mut GameState) -> Card {
        let hand = &game_state.hands[player];
        let mut best_card_index = 0usize;
        //print!("( ");
        match game_state.round.suit {
            Some(suit) => {
                let mut best_score = (false, std::f64::NEG_INFINITY);
                for (i, card) in hand.iter().enumerate() {
                    let value: f64 = self.value(player, &game_state, &card);
                    // print!("{}:{} ", card, (value * 100.0) as i32);
                    /*
                    let value: f64 = if player % 2 == 1 {
                        table.value(player, &game_state, &card)
                    } else {
                        random()
                    };
                    */
                    let is_suit = card.suit == suit;
                    if (is_suit, value) > best_score {
                        best_card_index = i;
                        best_score = (is_suit, value);
                    }
                }
                print!("({:.3}) ", (best_score.1 - 0.5) * 2.0 * 44.0);
            }
            None => {
                let mut best_score = std::f64::NEG_INFINITY;
                for (i, card) in hand.iter().enumerate() {
                    let value: f64 = self.value(player, &game_state, &card);
                    // print!("{}:{} ", card, (value * 10000.0) as i32);
                    /*
                    let value: f64 = if player % 2 == 1 {
                        table.value(player, &game_state, &card)
                    } else {
                        random()
                    };
                    */
                    if value > best_score {
                        best_card_index = i;
                        best_score = value;
                    }
                }
                print!("({:.3}) ", (best_score - 0.5) * 2.0 * 44.0);
            }
        }
        //print!(") ");
        game_state.hands[player].swap_remove(best_card_index)
    }
}

#[derive(Clone)]
struct GameState {
    hands: [Vec<Card>; NUM_PLAYERS],
    played_cards: Vec<Card>,
    trump: Card,
    round: Round,
    score: (u8, u8),
}

impl GameState {
    fn encode(&self, player: usize) -> Vec<f64> {
        let mut v = vec![0.0; NUM_CARDS * 8];
        v[self.trump.id as usize] = 1.0;
        let hand = &self.hands[player];
        for card in hand.iter() {
            v[NUM_CARDS + card.id as usize] = 1.0;
        }
        for card in self.played_cards.iter() {
            v[NUM_CARDS * 2 + card.id as usize] = 1.0;
        }
        for player in 0..NUM_PLAYERS {
            for card in self.round.played_cards[player].iter() {
                v[NUM_CARDS * (3 + player) + card.id as usize] = 1.0;
            }
        }
        if let Some(card) = self.round.played_cards[player] {
            v[NUM_CARDS * 7 + card.id as usize] = 1.0;
        }
        v
    }
}

#[derive(Clone)]
struct Round {
    suit: Option<CardSuit>,
    played_cards: [Option<Card>; NUM_PLAYERS],
}

impl Round {
    fn new() -> Round {
        Round {
            suit: None,
            played_cards: [None; NUM_PLAYERS]
        }
    }

    fn winner(&self, game_state: &GameState) -> usize {
        let mut winner = 0usize;
        let mut winner_value = 0u8;
        for (player, card) in self.played_cards.iter().enumerate() {
            if let Some(card) = card {
                let value = card_value(card, self.suit.expect("Round must have a suit to determine winner"), game_state.trump.suit);
                if value > winner_value {
                    winner = player;
                    winner_value = value;
                }
            }
        }
        winner
    }

    fn points(&self) -> u8 {
        let mut points = 0;
        for card in self.played_cards.iter() {
            if let Some(card) = card {
                points += card.points();
            }
        }
        points
    }
}

#[derive(Copy, Clone, PartialEq)]
enum CardSuit {
    Spades = 0,
    Hearts = 1,
    Diamonds = 2,
    Clubs = 3,
}

#[derive(Copy, Clone)]
struct Card {
    id: u8,
    value: u8,
    suit: CardSuit,
}

impl std::fmt::Display for CardSuit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let symbol = match self {
            CardSuit::Spades => '♠',
            CardSuit::Hearts => '♥',
            CardSuit::Diamonds => '♦',
            CardSuit::Clubs => '♣',
        };
        write!(f, "{}", symbol)
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let symbol = match self.value {
            0 => '2',
            1 => '3',
            2 => '4',
            3 => '5',
            4 => '6',
            5 => 'Q',
            6 => 'J',
            7 => 'K',
            8 => '7',
            9 => 'A',
            _ => unreachable!(),
        };
        write!(f, "{}{}", symbol, self.suit)
    }
}

impl Card {
    fn new(card_id: u8) -> Card {
        Card {
            id: card_id,
            value: card_id % NUM_CARDS_PER_SUIT as u8,
            suit: match card_id / NUM_CARDS_PER_SUIT as u8 {
                0 => CardSuit::Spades,
                1 => CardSuit::Hearts,
                2 => CardSuit::Diamonds,
                3 => CardSuit::Clubs,
                _ => unreachable!(),
            },
        }
    }

    fn points(&self) -> u8 {
        match self.value {
            5 => 2,
            6 => 3,
            7 => 4,
            8 => 10,
            9 => 11,
            _ => 0
        }
    }
}

type Deck = Vec<Card>;

fn game(deck: &mut Deck, table: &mut SuecaQLTable) -> (u8, u8) {
    deck_shuffle(deck);
    // Initialize game_state
    let mut game_state = GameState {
        hands: [Vec::with_capacity(NUM_CARDS_PER_HAND),
                Vec::with_capacity(NUM_CARDS_PER_HAND),
                Vec::with_capacity(NUM_CARDS_PER_HAND),
                Vec::with_capacity(10)],
        played_cards: Vec::with_capacity(NUM_CARDS),
        trump: deck[0],
        round: Round::new(),
        score: (0, 0),
    };
    println!("Trump: {}", game_state.trump);

    // Set hands of players
    for (i, card) in deck.iter().enumerate() {
        let player_id = i / NUM_CARDS_PER_HAND;
        game_state.hands[player_id].push(card.clone());
    }

    let mut initial_player = 0usize;

    for _ in 0..NUM_CARDS_PER_HAND {
        print!("Round: ");
        game_state.round = Round::new();
        let mut game_state_history: [Option<GameState>; NUM_PLAYERS] = [None, None, None, None];

        let mut player = initial_player;
        {
            game_state_history[player] = Some(game_state.clone());
            let card = table.choose_card(initial_player, &mut game_state);
            print!("{}|{} ", player, card);
            game_state.round.suit = Some(card.suit);
            update_round(player, card, &mut game_state);
            player = (player + 1) % NUM_PLAYERS;
        }
        while player != initial_player {
            game_state_history[player] = Some(game_state.clone());
            let card = table.choose_card(player, &mut game_state);
            print!("{}|{} ", player, card);
            update_round(player, card, &mut game_state);
            player = (player + 1) % NUM_PLAYERS;
        }
        print!("\t|");
        for i in 0..NUM_PLAYERS {
            let hand = &game_state.hands[(initial_player + i) / NUM_PLAYERS];
            for card in hand.iter() {
                print!("{} ", card);
            }
            print!("|");
        }
        let winner = update_score(&mut game_state);
        table.update(&game_state, game_state_history);
        initial_player = winner.clone();
    }
    println!("{} - {}", game_state.score.0, game_state.score.1);
    game_state.score
}

fn update_score(game_state: &mut GameState) -> usize {
    let winner = game_state.round.winner(&game_state);
    let round_points = game_state.round.points();
    if winner % 2 == 0 {
        game_state.score.0 += round_points;
    } else {
        game_state.score.1 += round_points;
    }
    println!("{}", round_points);
    winner
}

fn update_round(player: usize, card: Card, game_state: &mut GameState) {
    game_state.played_cards.push(card);
    game_state.round.played_cards[player] = Some(card);

}

fn card_value(card: &Card, suit: CardSuit, trump: CardSuit) -> u8 {
    const TRUMP_VALUE : u8 = 64;
    if card.suit == trump {
        TRUMP_VALUE + card.value
     } else if card.suit == suit {
        card.value
    } else {
        0
    }
}

fn generate_deck() -> Deck {
    let mut deck: Deck = Vec::with_capacity(NUM_CARDS);
    for i in 0..NUM_CARDS {
        deck.push(Card::new(i as u8));
    }
    deck
}

fn deck_shuffle(deck: &mut Deck) {
    let mut rng = thread_rng();
    let uniform = Uniform::new(0usize, deck.len());
    for i in 0..(deck.len() - 2) {
        let j: usize = uniform.sample(&mut rng);
        let v = deck[j];
        deck[j] = deck[i];
        deck[i] = v;
    }
    Uniform::new(0, deck.len());
}