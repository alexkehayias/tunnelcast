#![allow(dead_code)]

use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug)]
enum Action {
    Draw,
    PlayCard(u32),
    EndTurn,
    EnemyTurn
}

#[derive(Debug)]
enum Attribute {
    Shields,
    Weaponry,
    Power,
}

#[derive(Debug)]
enum Entity {
    Player,
    Enemy,
    RandomEnemy
}

#[derive(Debug)]
enum Effect{
    Increase(Entity, Attribute, u32),
    Decrease(Entity, Attribute, u32),
    // TODO AddCard, RemoveCard
}

#[derive(Debug)]
struct Card {
    name: &'static str,
    effects: Vec<Effect>
}

#[derive(Debug)]
struct GameState {
    deck: Vec<Card>,
    hand: Vec<Card>,
    shields: u32,
    power: u32,
    weaponry: u32,
    action: Action,
    // buffs: Vec<Buff>
}

impl GameState {
    fn new(deck: Vec<Card>, hand: Vec<Card>) -> GameState {
        GameState {
            deck,
            hand,
            shields: 0,
            power: 0,
            weaponry: 0,
            action: Action::Draw,
        }
    }
}

fn tick(game: &mut GameState) -> &mut GameState {
    match game.action {
        Action::Draw => {
            // TODO calculate how many to draw
            if let Some(card) = game.deck.pop() {
                game.hand.push(card);
            }
            game
        },
        Action::PlayCard(idx) => {
            let card = &game.hand[idx as usize];

            for fx in card.effects.iter() {
                println!("Effect: {:?}", fx);
                match fx {
                    Effect::Increase(_ent, attr, value) => {
                        match attr {
                            Attribute::Shields => {
                                game.shields += value;
                            }
                            _ => ()
                        }
                    },
                    _ => ()
                }
            }

            game
        }
        _ => game
    }
}

fn shuffle_deck(deck: &mut Vec<Card>) -> &mut Vec<Card> {
    let mut rng = thread_rng();
    deck.shuffle(&mut rng);
    deck
}

fn main() {
    let card_shields = Card {
        name: "Shields",
        effects: vec![Effect::Increase(Entity::Player, Attribute::Shields, 5)]
    };

    let card_phasers = Card {
        name: "Phasers",
        effects: vec![Effect::Increase(Entity::Player, Attribute::Shields, 5)]
    };

    let mut init_deck = vec![
        card_shields,
        card_phasers,
    ];
    shuffle_deck(&mut init_deck);

    let init_hand = vec![];

    let mut game = GameState::new(init_deck, init_hand);

    // Draw a hand
    println!("State: {:?}", tick(&mut game));

    // Play a card
    game.action = Action::PlayCard(0);
    println!("State: {:?}", tick(&mut game));
}
