
fn main() {
    println!("ola");
    game();
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

struct GameState {
    hands: [[bool; 40]; 4],
    played_cards: [bool; 40],
}

enum CardValue {
    2 = 0,
    3 = 1,
    A = 9
}

enum CardSuit {
    Hearts = 0,
    Diamonds = 1,
    Clubs = 2,
    Spades = 3
}

struct Card {
    value: u8,
    suit: CardSuit
}

fn game() {
    const NUM_PLAYERS: i32 = 4;
    let game_state: GameState = GameState {
        hands: [[false; 40]; 4],
        played_cards: [false; 40],        
    };
}

fn draw_cards(game_state: GameState) {
    const NUM_CARDS: i32 = 40;
    let num_cards_player = [10; 4];
    for i in 0..NUM_CARDS {
        
    }
}