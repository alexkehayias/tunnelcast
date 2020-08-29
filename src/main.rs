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

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum Attribute {
    Shields,
    Weaponry,
    Power,
    Hull,
}

trait Entity: std::fmt::Debug {
    fn get_state(&mut self) -> &mut State;
}

// For now, combining entities with state for simplicity.
#[derive(Debug)]
struct Player {
    state: State
}
impl Entity for Player {
    fn get_state(&mut self) -> &mut State {
        &mut self.state
    }
}

#[derive(Debug)]
struct Enemy {
    state: State,
}
impl Entity for Enemy {
    fn get_state(&mut self) -> &mut State {
        &mut self.state
    }
}

trait Effect: std::fmt::Debug {
    fn calculate(&self, game: &GameState, ent_idx: i32) -> State;
}

#[derive(Debug)]
struct IncreaseShields;

impl Effect for IncreaseShields {
    fn calculate(&self, _game: &GameState, _ent_idx: i32) -> State {
        let mut m = State::new();
        m.insert(Attribute::Shields, 1i32);

        m
    }
}

#[derive(Debug)]
struct DamageHull;

impl Effect for DamageHull {
    fn calculate(&self, _game: &GameState, _ent_idx: i32) -> State {
        let mut m = State::new();
        m.insert(Attribute::Hull, -1i32);

        m
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

type State = HashMap<Attribute, i32>;
type StateChange = (i32, HashMap<Attribute, i32>);

impl GameState {
    fn new(cards: CardCollection, deck: Vec<CardId>) -> GameState {
        GameState {
            cards,
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

    fn add_entity(&mut self, entity: Box<dyn Entity>) -> &mut Self {
        self.entities.push(entity);
        self
    }

    fn apply_effect(&mut self, state_change: StateChange) {
        println!("Applying state change {:?}", state_change);

        let (ent_idx, state) = state_change;
        let entity_state = self.entities.get_mut(ent_idx as usize)
            .expect("Failed ot get entity")
            .get_state();

        for (k, v) in state.iter() {
            *entity_state.entry(*k).or_insert(0) += v;
        }
    }
}

/// Progress the game forward one tick
// TODO implement a state machine and transitions between stages
fn tick(game: &mut GameState) -> &mut GameState {
    match game.action {
        Action::Draw => {
            // TODO calculate how many to draw
            if let Some(card) = game.draw.pop() {
                game.hand.push(card);
            }
            // TODO if there are no cards in the draw pile, move the
            // discard pile to the draw pile and shuffle
        },
        Action::PlayCard(ent_idx, card_idx) => {
            let card_id = &game.hand[card_idx as usize];
            let card = &game.cards
                .get(card_id)
                .unwrap_or_else(|| panic!("Could not find card with ID {:?}", card_id));

            let mut accum = State::new();
            for fx in &card.effects {
                println!("Effect: {:?}", fx);
                let effect = fx.calculate(&game, ent_idx);

                // Merge the effect by summing it with any existing
                // value in the accumumulator
                for (k, v) in effect.iter() {
                    if let Some(val) = accum.get_mut(k) {
                        *val += v;
                    } else {
                        accum.insert(*k, *v);
                    };
                }
            }

            // Move the card to the discard pile
            game.discard.push(*card_id);
            game.hand.remove(card_idx as usize);

            // This needs to happen after discard otherwise there is a
            // borrow error because card_id still immutably borrows
            // GameState and apply_effect needs a mutable reference
            game.apply_effect((ent_idx, accum));
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
    fn apply_effects() {
        // Initialize game state for the test
        let cards = CardCollection::new();
        let init_deck = vec![];
        let mut game = GameState::new(cards, init_deck);

        // Add a player entity
        let player_id = 0i32;
        let mut s = State::new();
        s.insert(Attribute::Hull, 10);
        s.insert(Attribute::Shields, 10);
        let player = Player { state: s };
        game.add_entity(Box::new(player));

        // We'll test the shields card effects are applied correctly
        let card = Card {
            id: CardId::Shields,
            name: "Shields",
            effects: vec![Box::new(IncreaseShields {})]
        };

        // Apply state change for the card
        let state_change = card.effects[0]
            .calculate(&game, player_id);
        game.apply_effect((player_id, state_change));

        assert_eq!(
            game.entities[player_id as usize]
                .get_state()
                .get(&Attribute::Shields)
                .unwrap(),
            &11i32
        )
    }

    #[test]
    fn integration() {
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
        let mut s = State::new();
        s.insert(Attribute::Hull, 10);
        s.insert(Attribute::Shields, 10);
        let enemy = Enemy { state: s };
        game.add_entity(Box::new(enemy));

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
