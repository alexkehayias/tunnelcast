//! Implements a finite state machine to represent different GUI
//! states.
use crate::engine::{Target, EntityId};


struct GuiStateMachine<T> {
    state: T
}

impl GuiStateMachine<Combat> {
    fn new(enemy_id: EntityId) -> Self {
        GuiStateMachine {
            state: Combat {
                enemy_id,
                target_id: None
            }
        }
    }
}

struct Combat {
    enemy_id: EntityId,
    target_id: Option<EntityId>
}

struct Targeting {
    enemy_id: EntityId,
    target_type: Target,
    targets: Vec<EntityId>,
    target_id: Option<EntityId>
}

impl From<GuiStateMachine<Combat>> for GuiStateMachine<Targeting> {
    fn from(val: GuiStateMachine<Combat>) -> GuiStateMachine<Targeting> {
        GuiStateMachine {
            state: Targeting {
                enemy_id: val.state.enemy_id,
                target_type: Target::Single,
                targets: vec![val.state.enemy_id],
                target_id: None
            }
        }
    }
}

impl From<GuiStateMachine<Targeting>> for GuiStateMachine<Combat> {
    fn from(val: GuiStateMachine<Targeting>) -> GuiStateMachine<Combat> {
        GuiStateMachine {
            state: Combat {
                enemy_id: val.state.enemy_id,
                target_id: Some(val.state.target_id.expect("A target entity ID should have been selected at this point")),
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
        targeting_state.state.target_id = Some(enemy_id);
        let combat_state_with_target = GuiStateMachine::<Combat>::from(targeting_state);

        assert_eq!(
            combat_state_with_target.state.target_id.unwrap(),
            enemy_id
        );
    }
}
