//! Implements a finite state machine to represent different GUI
//! states.
use crate::engine::{Target, EntityId};

/// A collection of shared state between different transitions. Useful
/// so you don't need to duplicate the same attributes across multiple
/// states.
pub struct SharedState {
    enemy_id: EntityId,
    target_id: Option<EntityId>,
}

pub struct GuiStateMachine<T> {
    pub state: T
}

impl GuiStateMachine<Combat> {
    pub fn new(enemy_id: EntityId) -> Self {
        GuiStateMachine {
            state: Combat {
                shared_state: SharedState {
                    enemy_id,
                    target_id: None
                }
            }
        }
    }
}

pub struct Combat {
    shared_state: SharedState,
}

pub struct Targeting {
    shared_state: SharedState,
    target_type: Target,
    targets: Vec<EntityId>,
}

impl From<GuiStateMachine<Combat>> for GuiStateMachine<Targeting> {
    fn from(val: GuiStateMachine<Combat>) -> GuiStateMachine<Targeting> {
        let targets = vec![val.state.shared_state.enemy_id];
        GuiStateMachine {
            state: Targeting {
                shared_state: val.state.shared_state,
                target_type: Target::Single,
                targets: targets,
            }
        }
    }
}

impl From<&GuiStateMachine<Combat>> for GuiStateMachine<Targeting> {
    fn from(val: &GuiStateMachine<Combat>) -> GuiStateMachine<Targeting> {
        let enemy_id = val.state.shared_state.enemy_id;
        let target_id = val.state.shared_state.target_id;
        let targets = vec![enemy_id];

        GuiStateMachine {
            state: Targeting {
                // HACK: Constructing the shared state manually to
                // avoid a borrowck error because we're using
                // state.shared_state to create the vector of targets
                shared_state: SharedState {
                    enemy_id,
                    target_id
                },
                target_type: Target::Single,
                targets: targets
            }
        }
    }
}

impl From<GuiStateMachine<Targeting>> for GuiStateMachine<Combat> {
    fn from(val: GuiStateMachine<Targeting>) -> GuiStateMachine<Combat> {

        // TODO Is there a way to make this not a runtime exception?
        // Assert that we have a target ID otherwise this is an
        // invalid transition.
        val.state.shared_state.target_id.expect("A target entity ID should have been selected at this point");

        GuiStateMachine {
            state: Combat {
                shared_state: val.state.shared_state
            }
        }
    }
}

mod test_gui_state_machine {
    use super::*;

    #[test]
    fn test_integration() {
        let enemy_id = 1;
        let combat_state = GuiStateMachine::<Combat>::new(1);

        // Simulate the user selecting a target
        let mut targeting_state = GuiStateMachine::<Targeting>::from(combat_state);
        targeting_state.state.shared_state.target_id = Some(enemy_id);
        let combat_state_with_target = GuiStateMachine::<Combat>::from(targeting_state);

        assert_eq!(
            combat_state_with_target.state.shared_state.target_id.unwrap(),
            enemy_id
        );
    }
}
