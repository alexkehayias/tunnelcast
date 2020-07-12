#![allow(dead_code)]

use std::collections::HashMap;
use std::cmp::{Eq, PartialEq};
use std::hash::Hash;

use rand::seq::SliceRandom;
use rand::thread_rng;
use lazy_static::lazy_static;

#[derive(Debug, Eq, PartialEq, Hash)]
enum CardId {
    Shields,
    Phasers,
}

lazy_static! {
    static ref CARDS: HashMap<CardId, Card> = {
        let mut m = HashMap::new();
        let card_shields = Card {
            name: "Shields",
            effects: vec![Effect::Increase(Entity::Player, Attribute::Shields, 5)]
        };

        let card_phasers = Card {
            name: "Phasers",
            effects: vec![Effect::Increase(Entity::Player, Attribute::Shields, 5)]
        };

        m.insert(CardId::Shields, card_shields);
        m.insert(CardId::Phasers, card_phasers);

        m
    };
}


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
    deck: Vec<CardId>,
    hand: Vec<CardId>,
    shields: u32,
    power: u32,
    weaponry: u32,
    action: Action,
    // buffs: Vec<Buff>
}

impl GameState {
    fn new(deck: Vec<CardId>, hand: Vec<CardId>) -> GameState {
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
        },
        Action::PlayCard(idx) => {
            let card_id = &game.hand[idx as usize];
            let card = &CARDS[card_id];

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
        }
        _ => ()
    }

    game
}

fn shuffle_deck(deck: &mut Vec<CardId>) -> &mut Vec<CardId> {
    let mut rng = thread_rng();
    deck.shuffle(&mut rng);
    deck
}

fn main() {

    let mut init_deck = vec![
        CardId::Shields,
        CardId::Phasers,
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
