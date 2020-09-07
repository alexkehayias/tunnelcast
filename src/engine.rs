#![allow(dead_code)]

use std::collections::HashMap;
use std::cmp::{Eq, PartialEq};
use std::hash::Hash;

use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum CardId {
    Shields,
    Phasers,
}

#[derive(Debug)]
pub enum Action {
    None,
    Draw,
    PlayCard(EntityId, i32),
    BeginTurn,
    EndTurn,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Attribute {
    Shields,
    Hull,
}

pub type EntityId = u32;

pub trait Entity: std::fmt::Debug {
    fn get_state(&mut self) -> &mut State;
}

fn gen_id() -> EntityId {
    rand::random::<u32>()
}

// For now, combining entities with state for simplicity.
#[derive(Debug)]
pub struct Player {
    pub state: State
}
impl Entity for Player {
    fn get_state(&mut self) -> &mut State {
        &mut self.state
    }
}

#[derive(Debug)]
pub struct Enemy {
    pub state: State,
}
impl Entity for Enemy {
    fn get_state(&mut self) -> &mut State {
        &mut self.state
    }
}

pub trait Effect: std::fmt::Debug {
    fn calculate(&self, game: &GameState, ent_id: EntityId) -> State;
}

#[derive(Debug)]
pub struct IncreaseShields;

impl Effect for IncreaseShields {
    fn calculate(&self, _game: &GameState, _ent_id: EntityId) -> State {
        let mut m = State::new();
        m.insert(Attribute::Shields, 1i32);

        m
    }
}

#[derive(Debug)]
pub struct DamageHull;

impl Effect for DamageHull {
    fn calculate(&self, _game: &GameState, _ent_id: EntityId) -> State {
        let mut m = State::new();
        m.insert(Attribute::Hull, -1i32);

        m
    }
}

#[derive(Debug)]
pub struct Card {
    pub id: CardId,
    pub name: &'static str,
    pub effects: Vec<Box<dyn Effect>>
}

#[derive(Debug)]
pub struct GameState {
    pub cards: CardCollection,
    pub draw: Vec<CardId>,
    pub hand: Vec<CardId>,
    pub discard: Vec<CardId>,
    pub action: Action,
    pub entities: Vec<EntityId>,
    pub entity_state: HashMap<EntityId, Box<dyn Entity>>,
}

pub type State = HashMap<Attribute, i32>;
type StateChange = (EntityId, HashMap<Attribute, i32>);

impl GameState {
    pub fn new(cards: CardCollection, deck: Vec<CardId>) -> GameState {
        GameState {
            cards,
            draw: deck,
            hand: vec![],
            discard: vec![],
            action: Action::None,
            entities: vec![],
            entity_state: HashMap::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Box<dyn Entity>) -> EntityId {
        let entity_id = gen_id();
        self.entities.push(entity_id);
        self.entity_state.insert(entity_id, entity);
        entity_id
    }

    fn remove_entity(&mut self, entity_id: &EntityId) {
        let index = self.entities.iter()
            .position(|x| x == entity_id)
            .expect("EntityId not found");
        self.entities.remove(index);
        self.entity_state.remove(entity_id);
    }

    fn apply_effect(&mut self, state_change: StateChange) {
        println!("Applying state change {:?}", state_change);

        let (entity_id, state) = state_change;
        let entity_state = self.entity_state.get_mut(&entity_id)
            .expect("Failed ot get entity")
            .get_state();

        for (k, v) in state.iter() {
            *entity_state.entry(*k).or_insert(0) += v;
        }

        // Removing entity from the game if hull drops to zero
        if entity_state.get(&Attribute::Hull).unwrap() <= &0 {
            self.remove_entity(&entity_id);
        }
    }
}

/// Progress the game forward one tick
// TODO implement a state machine for taking turns and transition
// between stages
// TODO maybe this should emit events that the UI layer
// can interpret e.g. discard pile moved to draw pile
pub fn tick(game: &mut GameState) -> &mut GameState {
    match game.action {
        Action::None => (),
        Action::Draw => {
            // If draw pile is empty, shuffle and move discard pile
            // into the draw pile.
            if game.draw.iter().count() == 0 {
                shuffle_deck(&mut game.discard);
                game.draw.append(&mut game.discard);
            }

            if let Some(card) = game.draw.pop() {
                game.hand.push(card);
            };
        },
        Action::PlayCard(target_ent_idx, card_idx) => {
            let card_id = &game.hand[card_idx as usize];
            let card = &game.cards
                .get(card_id)
                .unwrap_or_else(|| panic!("Could not find card with ID {:?}", card_id));

            let mut accum = State::new();
            for fx in &card.effects {
                println!("Effect: {:?}", fx);
                let effect = fx.calculate(&game, target_ent_idx);

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
            game.apply_effect((target_ent_idx, accum));
        },
        Action::BeginTurn => {
            draw_hand(game, 4);
        },
        Action::EndTurn => {
            discard_hand(game);
        }
    }

    game
}

pub fn shuffle_deck(deck: &mut Vec<CardId>) -> &mut Vec<CardId> {
    let mut rng = thread_rng();
    deck.shuffle(&mut rng);
    deck
}

/// Move `count` cards from the draw pile to the hand
pub fn draw_hand(game: &mut GameState, count: i8) -> &mut GameState {
    for _ in 0..count {
        if let Some(card_id) = game.draw.pop() {
            game.hand.push(card_id);
        }
    }

    game
}

/// Move all cards from hand to the discard pile
fn discard_hand(game: &mut GameState) -> &mut GameState {
    // TODO handle cards that persist between turns
    game.discard.append(&mut game.hand);
    game
}

#[derive(Debug)]
pub struct CardCollection {
    inner: HashMap<CardId, Card>
}

impl CardCollection {
    pub fn new() -> Self {
        Self {inner: HashMap::new()}
    }

    pub fn insert(&mut self, card: Card) {
        self.inner.insert(card.id, card);
    }

    pub fn get(&self, card_id: &CardId) -> Option<&Card> {
        self.inner.get(card_id)
    }
}


mod test_game {
    use super::*;

    #[test]
    fn test_draw_hand() {
        // Initialize game state for the test
        let cards = CardCollection::new();
        let init_deck = vec![];
        let mut game = GameState::new(cards, init_deck);

        // Drawing a hand with an empty deck should not panic
        draw_hand(&mut game, 4);
        assert_eq!(game.hand, vec![], "Hand should be empty");

        // Try with a draw pile of three cards and try to draw four
        let expected_hand = vec![
            CardId::Phasers,
            CardId::Phasers,
            CardId::Phasers
        ];
        game.draw = expected_hand.clone();
        draw_hand(&mut game, 4);
        assert_eq!(expected_hand, game.hand);
        assert!(game.draw.is_empty(), "Draw pile should be empty");
    }

    #[test]
    fn test_discard_hand() {
        // Initialize game state for the test
        let cards = CardCollection::new();
        let init_deck = vec![];
        let mut game = GameState::new(cards, init_deck);

        // Try with a draw pile of three cards and try to draw four
        game.hand = vec![
            CardId::Phasers,
            CardId::Phasers,
        ];
        discard_hand(&mut game);
        assert!(game.hand.is_empty(), "Hand should be empty");
        assert_eq!(
            vec![CardId::Phasers,CardId::Phasers],
            game.discard,
            "Cards from the hand should all be in the discard pile"
        )
    }

    #[test]
    fn test_apply_effects() {
        // Initialize game state for the test
        let cards = CardCollection::new();
        let init_deck = vec![];
        let mut game = GameState::new(cards, init_deck);

        // Add a player entity
        let mut s = State::new();
        s.insert(Attribute::Hull, 10);
        s.insert(Attribute::Shields, 10);
        let player = Player { state: s };
        let player_id = game.add_entity(Box::new(player));

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
            game.entity_state.get_mut(&player_id)
                .unwrap()
                .get_state()
                .get(&Attribute::Shields)
                .unwrap(),
            &11i32
        )
    }

    #[test]
    fn test_integration() {
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

        // Add an enemy
        let mut s = State::new();
        s.insert(Attribute::Hull, 10);
        s.insert(Attribute::Shields, 10);
        let enemy = Enemy { state: s };
        let enemy_id = game.add_entity(Box::new(enemy));

        // Run through a turn to make sure it works
        game.action = Action::BeginTurn;
        println!("State: {:?}", tick(&mut game));

        game.action = Action::PlayCard(enemy_id, 0);
        println!("State: {:?}", tick(&mut game));

        game.action = Action::PlayCard(enemy_id, 0);
        println!("State: {:?}", tick(&mut game));

        game.action = Action::EndTurn;
        println!("State: {:?}", tick(&mut game));
    }
}
