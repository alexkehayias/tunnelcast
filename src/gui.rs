//! Implements a finite state machine to represent different GUI
//! states. Does not ensure that transitions happen only once because
//! the implementation for From does not consume the value.
//!
//! See [this blog post](https://hoverbear.org/blog/rust-state-machine-pattern/)
//! for more about this design
use crate::engine::EntityId;

/// A collection of shared state between different transitions. Useful
/// so you don't need to duplicate the same attributes across multiple
/// states.
pub struct SharedState {}

pub struct GuiStateMachine<T> {
    pub state: T,
}

pub trait TransitionFrom<T> {
    type Args;
    fn transition_from(state: T, args: Self::Args) -> Self;
}

pub trait GuiState {
    type Args;
    fn new(args: Self::Args) -> Self;
}

pub struct Combat {
    pub shared_state: SharedState,
    pub enemy_id: EntityId,
}

impl Combat {
    pub fn new(enemy_id: EntityId) -> Self {
        Combat {
            shared_state: SharedState {},
            enemy_id,
        }
    }
}

impl GuiStateMachine<Combat> {
    pub fn new(enemy_id: EntityId) -> Self {
        GuiStateMachine {
            state: Combat::new(enemy_id)
        }
    }
}

pub struct PlayCard {
    pub shared_state: SharedState,
    pub card_idx: u32,
}

pub struct PlayCardArgs {
    pub card_idx: u32,
}

impl TransitionFrom<&GuiStateMachine<Combat>> for GuiStateMachine<PlayCard> {
    type Args = PlayCardArgs;

    fn transition_from(
        _fsm: &GuiStateMachine<Combat>,
        args: PlayCardArgs,
    ) -> GuiStateMachine<PlayCard> {
        GuiStateMachine {
            state: PlayCard {
                shared_state: SharedState {},
                card_idx: args.card_idx,
            },
        }
    }
}

pub struct TargetSelect {
    pub shared_state: SharedState,
    pub targets: Vec<EntityId>,
    pub card_idx: u32,
}

pub struct TargetSelectArgs {
    pub targets: Vec<EntityId>,
    pub card_idx: u32,
}

impl TransitionFrom<&GuiStateMachine<PlayCard>> for GuiStateMachine<TargetSelect> {
    type Args = TargetSelectArgs;

    fn transition_from(
        _fsm: &GuiStateMachine<PlayCard>,
        args: TargetSelectArgs,
    ) -> GuiStateMachine<TargetSelect> {
        GuiStateMachine {
            state: TargetSelect {
                shared_state: SharedState {},
                targets: args.targets,
                card_idx: args.card_idx,
            },
        }
    }
}

pub struct TargetSelectComplete {
    pub shared_state: SharedState,
    /// The selected target for the played card
    pub target: EntityId,
    pub card_idx: u32,
}

pub struct TargetSelectCompleteArgs {
    pub target: EntityId,
}

impl TransitionFrom<&GuiStateMachine<TargetSelect>> for GuiStateMachine<TargetSelectComplete> {
    type Args = TargetSelectCompleteArgs;

    fn transition_from(
        fsm: &GuiStateMachine<TargetSelect>,
        args: TargetSelectCompleteArgs,
    ) -> GuiStateMachine<TargetSelectComplete> {
        GuiStateMachine {
            state: TargetSelectComplete {
                shared_state: SharedState {},
                target: args.target,
                card_idx: fsm.state.card_idx,
            },
        }
    }
}

#[cfg(test)]
mod test_gui_state_machine {
    use super::*;

    #[test]
    fn test_transitions() {
        let card_idx = 0;
        let enemy_id = 1;

        // Initial state
        let combat_state = GuiStateMachine {
            state: Combat {
                shared_state: SharedState {},
                enemy_id,
            },
        };

        // Simulate playing a card
        let play_card_state =
            GuiStateMachine::<PlayCard>::transition_from(&combat_state, PlayCardArgs { card_idx });

        // Simulate the user selecting a target
        let targeting_state = GuiStateMachine::<TargetSelect>::transition_from(
            &play_card_state,
            TargetSelectArgs {
                targets: vec![enemy_id],
                card_idx,
            },
        );

        // Simulate choosing enemy as target
        let target_select_complete_state = GuiStateMachine::<TargetSelectComplete>::transition_from(
            &targeting_state,
            TargetSelectCompleteArgs { target: enemy_id },
        );

        assert_eq!(target_select_complete_state.state.target, enemy_id);
    }
}
