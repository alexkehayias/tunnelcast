#![allow(dead_code)]

use std::collections::HashMap;
use std::cmp::{Eq, PartialEq};
use std::hash::Hash;

use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
enum CardId {
    Shields,
    Phasers,
    Overdrive,
}

#[derive(Debug)]
enum Action {
    Draw,
    PlayCard(i32, i32),
    EndTurn,
    EnemyTurn
}

#[derive(Debug, Hash, Eq, PartialEq)]
enum Attribute {
    Shields,
    Weaponry,
    Power,
    Hull,
}

trait Entity: std::fmt::Debug {}

#[derive(Debug)]
struct Player;
impl Entity for Player {}

#[derive(Debug)]
struct Enemy {
    hull: i32,
    shields: i32,
}
impl Entity for Enemy {}

trait Effect: std::fmt::Debug {
    fn calculate(&self, game: &GameState, ent: &Box<dyn Entity>) -> StateChange;
}

#[derive(Debug)]
struct IncreaseShields;

impl Effect for IncreaseShields {
    fn calculate(&self, game: &GameState, ent: &Box<dyn Entity>) -> StateChange {
        let mut m = HashMap::new();
        m.insert(Attribute::Shields, 1u32);

        m
    }
}

#[derive(Debug)]
struct DamageHull;

impl Effect for DamageHull {
    fn calculate(&self, game: &GameState, ent: &Box<dyn Entity>) -> StateChange {
        println!("Damage hull!");
        // TODO: Find state of the entity and mutate it
        HashMap::new()
    }
}

#[derive(Debug)]
struct Card {
    id: CardId,
    name: &'static str,
    effects: Vec<Box<dyn Effect>>
}

#[derive(Debug)]
struct GameState {
    cards: CardCollection,
    deck: Vec<CardId>,
    draw: Vec<CardId>,
    hand: Vec<CardId>,
    discard: Vec<CardId>,
    shields: u32,
    power: u32,
    weaponry: u32,
    action: Action,
    entities: Vec<Box<dyn Entity>>,
    // buffs: Vec<Buff>
}

type StateChange = HashMap<Attribute, u32>;

impl GameState {
    fn new(cards: CardCollection, deck: Vec<CardId>) -> GameState {
        GameState {
            cards: cards,
            deck: deck.clone(),
            draw: deck,
            hand: vec![],
            discard: vec![],
            shields: 0,
            power: 0,
            weaponry: 0,
            action: Action::Draw,
            entities: vec![],
        }
    }

    fn calculate_effect(&self, effect: &Box<dyn Effect>, entity: &Box<dyn Entity>) -> StateChange {
        effect.calculate(self, entity)
    }

    fn apply_effect(&mut self, state_change: StateChange) {
        for (k, v) in state_change.iter() {
            println!("TODO: Apply the effect {:?} with value {:?}", k, v);
        }

    }
}

fn tick(game: &mut GameState) -> &mut GameState {
    match game.action {
        Action::Draw => {
            // TODO calculate how many to draw
            if let Some(card) = game.draw.pop() {
                game.hand.push(card);
            }
        },
        Action::PlayCard(ent_idx, card_idx) => {
            let entity = &game.entities[ent_idx as usize];
            let card_id = &game.hand[card_idx as usize];
            let card = &game.cards
                .get(card_id)
                .unwrap_or_else(|| panic!("Could not find card with ID {:?}", card_id));

            let mut accum = HashMap::new();
            for fx in &card.effects {
                println!("Effect: {:?}", fx);
                accum.extend(game.calculate_effect(fx, entity));
            }

            // Move the card to the discard pile
            game.discard.push(*card_id);
            game.hand.remove(card_idx as usize);

            // This needs to happen after discard otherwise there is a
            // borrow error because card_id still immutably borrows
            // GameState and apply_effect needs a mutable reference
            game.apply_effect(accum);
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

/// Move `count` cards from the draw pile to the hand
fn draw_hand(game: &mut GameState, count: i8) -> &mut GameState {
    for _ in 0..count {
        if let Some(card_id) = game.draw.pop() {
            game.hand.push(card_id);
        }
    }

    game
}

fn add_enemy(game: &mut GameState, enemy: Enemy) -> &mut GameState {
    game.entities.push(Box::new(enemy));
    game
}

#[derive(Debug)]
struct CardCollection {
    inner: HashMap<CardId, Card>
}

impl CardCollection {
    fn new() -> Self {
        Self {inner: HashMap::new()}
    }

    fn insert(&mut self, card: Card) {
        self.inner.insert(card.id, card);
    }

    fn get(&self, card_id: &CardId) -> Option<&Card> {
        self.inner.get(card_id)
    }
}

fn main() {
}


mod test_game {
    use super::*;

    #[test]
    fn it_works() {
        let mut cards = CardCollection::new();

        cards.insert(
            Card {
                id: CardId::Shields,
                name: "Shields",
                effects: vec![Box::new(IncreaseShields {})]
            }
        );

        cards.insert(
            Card {
                id: CardId::Phasers,
                name: "Phasers",
                effects: vec![Box::new(DamageHull {})]
            }
        );

        let mut init_deck = vec![
            CardId::Shields,
            CardId::Shields,
            CardId::Shields,
            CardId::Phasers,
            CardId::Phasers,
            CardId::Phasers,
        ];
        shuffle_deck(&mut init_deck);

        let mut game = GameState::new(cards, init_deck);

        // Add an opponent
        let enemy = Enemy { hull: 10, shields: 0 };
        add_enemy(&mut game, enemy);

        // Draw a hand
        draw_hand(&mut game, 4);
        println!("State: {:?}", tick(&mut game));

        // Play a card
        game.action = Action::PlayCard(0, 0);
        println!("State: {:?}", tick(&mut game));

        game.action = Action::PlayCard(0, 0);
        println!("State: {:?}", tick(&mut game));
    }
}
