//! Implements a finite state machine to represent different GUI
//! states. Does not ensure that transitions happen only once because
//! the implementation for From does not consume the value.
//!
//! See [this blog post](https://hoverbear.org/blog/rust-state-machine-pattern/)
//! for more about this design
use crate::engine::{Target, EntityId};

/// A collection of shared state between different transitions. Useful
/// so you don't need to duplicate the same attributes across multiple
/// states.
pub struct SharedState {
    /// Card played by index in the hand
    pub card_idx: Option<i32>,
    // TODO replace with a vector of enemies
    /// The enemy ID in the current combat stage
    pub enemy_id: EntityId,
}

pub struct GuiStateMachine<T> {
    pub state: T
}

pub struct GuiStateMachineWithArgs<T> {
    pub state: T,
}

pub trait TransitionFrom<T, R> {
    fn transition_from(state: T, args: R) -> Self;
}

pub trait GuiState {
    type Args;
    fn new(args: Self::Args) -> Self;
}

impl<T> GuiStateMachineWithArgs<T> where T: GuiState {
    pub fn new(state: T) -> Self {
        GuiStateMachineWithArgs { state }
    }
}

struct StateOne;
impl GuiState for StateOne {
    type Args = ();

    fn new(args: Self::Args) -> Self {
        Self {}
    }
}

struct StateTwo;
impl GuiState for StateTwo {
    type Args = ();

    fn new(args: Self::Args) -> Self {
        Self {}
    }
}

struct StateThree;
impl GuiState for StateThree {
    type Args = ();

    fn new(args: Self::Args) -> Self {
        Self {}
    }
}

impl TransitionFrom<GuiStateMachineWithArgs<StateOne>, ()> for GuiStateMachineWithArgs<StateTwo> {

    fn transition_from(state: GuiStateMachineWithArgs<StateOne>, args: ()) -> GuiStateMachineWithArgs<StateTwo> {
        GuiStateMachineWithArgs::new(StateTwo::new(args))
    }
}

impl TransitionFrom<GuiStateMachineWithArgs<StateTwo>, ()> for GuiStateMachineWithArgs<StateThree> {

    fn transition_from(state: GuiStateMachineWithArgs<StateTwo>, args: ()) -> GuiStateMachineWithArgs<StateThree> {
        GuiStateMachineWithArgs::new(StateThree::new(args))
    }
}

mod test_new_state_machine {
    use super::*;

    #[test]
    fn test_transitions() {
        let inner_state = StateOne::new(());
        let state_one = GuiStateMachineWithArgs::new(inner_state);
        let state_two = GuiStateMachineWithArgs::<StateTwo>::transition_from(state_one, ());
        // This won't compile because it's not a valid transition
        // GuiStateMachineWithArgs::<StateThree>::transition_from(state_one, ());
    }
}

impl GuiStateMachine<Combat> {
    pub fn new(card_idx: Option<i32>, enemy_id: EntityId) -> Self {
        GuiStateMachine {
            state: Combat {
                shared_state: SharedState {
                    card_idx,
                    enemy_id,
                }
            }
        }
    }
}

pub struct Combat {
    pub shared_state: SharedState,
}

pub struct TargetSelect {
    pub shared_state: SharedState,
    pub target_type: Target,
    pub targets: Vec<EntityId>
}

impl From<&GuiStateMachine<Combat>> for GuiStateMachine<TargetSelect> {
    fn from(val: &GuiStateMachine<Combat>) -> GuiStateMachine<TargetSelect> {
        let card_idx = val.state.shared_state.card_idx;
        let enemy_id = val.state.shared_state.enemy_id;
        let targets = vec![enemy_id];

        GuiStateMachine {
            state: TargetSelect {
                // HACK: Constructing the shared state manually to
                // avoid a borrowck error because we're using
                // state.shared_state to create the vector of targets
                shared_state: SharedState {
                    card_idx,
                    enemy_id,
                },
                target_type: Target::Single,
                targets: targets
            }
        }
    }
}


impl From<&GuiStateMachine<TargetSelect>> for GuiStateMachine<Combat> {
    fn from(val: &GuiStateMachine<TargetSelect>) -> GuiStateMachine<Combat> {
        let card_idx = val.state.shared_state.card_idx;
        let enemy_id = val.state.shared_state.enemy_id;

        GuiStateMachine {
            state: Combat {
                shared_state: SharedState {
                    card_idx,
                    enemy_id,
                }
            }
        }
    }
}

impl From<&mut GuiStateMachine<TargetSelect>> for GuiStateMachine<Combat> {
    fn from(val: &mut GuiStateMachine<TargetSelect>) -> GuiStateMachine<Combat> {
        let card_idx = val.state.shared_state.card_idx;
        let enemy_id = val.state.shared_state.enemy_id;

        GuiStateMachine {
            state: Combat {
                shared_state: SharedState {
                    card_idx,
                    enemy_id,
                }
            }
        }
    }
}

pub struct TargetSelectComplete {
    pub shared_state: SharedState,
    /// The selected target for the played card
    pub target: EntityId,
}

// mod test_gui_state_machine {
//     use super::*;

//     #[test]
//     fn test_integration() {
//         let card_idx = 0;
//         let enemy_id = 1;
//         let combat_state = GuiStateMachine::<Combat>::new(Some(card_idx), enemy_id);

//         // Simulate the user selecting a target
//         let targeting_state = GuiStateMachine::<TargetSelect>::from(&combat_state);

//         let target_select_complete_state = GuiStateMachine::<TargetSelectComplete>::from(&targeting_state);

//         targeting_state.state.shared_state.target_id = Some(enemy_id);
//         let combat_state_with_target = GuiStateMachine::<Combat>::from(&targeting_state);

//         assert_eq!(
//             combat_state_with_target.state.shared_state.target_id.unwrap(),
//             enemy_id
//         );
//     }
// }
