use backtrace::Backtrace;
use std::panic::{self, PanicInfo};
use std::{error::Error, io, time::Duration};

use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, Clear},
    Terminal,
};

mod engine;
mod event;
mod gui;

use engine::*;
use event::{Config, Event, Events};
use gui::*;

const SPACE_SHIP: &str = "
                           |-----------|
           i               |===========|
           |               |,---------.|                      __--~\\__--.
    #---,'----`-_   `n     |`---------'|    `n    `n     ,--~~  __-/~~--'_____.
       |~~~~~~~~~|---~---/=|___________|=\\---~-----~-----| .--~~  |  .__|     |
     -[|.--_. ===|#####|-| |@@@@|+-+@@@| |]=###|/-++++-[| ||||___+_.  | `===='-.
     -[|'==~'    |#####|-| |@@@@|+-+@@@| |]=###|\\-++++-[| ||||~~~+~'  | ,====.-'
       |_________|---u---\\=|~~~~~~~~~~~|=/---u-----u-----| '--__  |  '~~|     |
        \\       /=-   `    |,---------.|      `     `    `--__  ~~-\\__--.~~~~~'
----=:===\\     /           |`---------'|                      ~~--_/~~--'
      --<:\\___/--          |===========|
                           |-----------|
                           |___________|";

enum GuiState {
    Combat(GuiStateMachine<Combat>),
    TargetSelect(GuiStateMachine<TargetSelect>),
    TargetSelectComplete(GuiStateMachine<TargetSelectComplete>),
}

struct Game {
    game_state: GameState,
    gui_state: GuiState,
}

impl Game {
    fn init_state() -> GameState {
        let mut cards = CardCollection::new();

        cards.insert(Card {
            id: CardId::Shields,
            name: "Shields",
            effects: vec![Box::new(IncreaseShields {})],
            target: Target::Player,
        });

        cards.insert(Card {
            id: CardId::Phasers,
            name: "Phasers",
            effects: vec![Box::new(DamageHull {})],
            target: Target::Single,
        });

        let mut init_deck = vec![
            CardId::Shields,
            CardId::Shields,
            CardId::Shields,
            CardId::Phasers,
            CardId::Phasers,
            CardId::Phasers,
        ];
        shuffle_deck(&mut init_deck);

        let mut game_state = GameState::new(cards, init_deck);

        // Add player
        let mut s = State::new();
        s.insert(Attribute::Hull, 10);
        s.insert(Attribute::Shields, 10);
        let player = Player { name: String::from("Player"), state: s };
        let player_id = 1;
        game_state.add_entity(Some(player_id), Box::new(player));
        game_state.player = player_id;

        // Add an enemy
        let mut s = State::new();
        s.insert(Attribute::Hull, 10);
        s.insert(Attribute::Shields, 10);
        let enemy = Enemy { name: String::from("Battleship"), state: s };
        let enemy_id = 2;
        game_state.add_entity(Some(enemy_id), Box::new(enemy));
        game_state.enemy = Some(enemy_id);

        draw_hand(&mut game_state, 4);

        game_state
    }

    fn new() -> Self {
        let game_state = Self::init_state();
        let gui_state = GuiState::Combat(GuiStateMachine::<Combat>::new(game_state.enemy.unwrap()));

        Self {
            game_state,
            gui_state,
        }
    }

    fn handle_keyboard_input(&mut self, input: Key) -> &mut Self {
        match self.gui_state {
            GuiState::Combat(ref state) => {
                match input {
                    Key::Char('e') => {
                        self.game_state.action = Action::EndTurn;
                    }
                    Key::Char(num_char) => {
                        if ['1', '2', '3', '4', '5', '6', '7', '8', '9'].contains(&num_char)
                            && num_char.to_digit(10).unwrap()
                            <= self.game_state.hand.len() as u32
                        {
                            let card_idx = num_char.to_digit(10).unwrap() as usize;
                            let card_idx = (card_idx - 1) as u32; // Convert to vector index
                            let card_id = self.game_state.hand[card_idx as usize];
                            let selected_card = self.game_state.cards.get(&card_id).unwrap();

                            let next_gui_state = GuiStateMachine::<PlayCard>::transition_from(
                                state,
                                PlayCardArgs { card_idx },
                            );

                            // Determine the target of the card or
                            // prompt the user
                            match selected_card.target {
                                Target::Player => {
                                    self.game_state.action = Action::PlayCard(
                                        self.game_state.player,
                                        card_idx as i32,
                                    );
                                }
                                Target::Single => {
                                    // TODO If there is only a single
                                    // enemy then skip the transition
                                    let enemy = self
                                        .game_state
                                        .enemy
                                        .expect("Can't target if there are no enemies");
                                    let next_gui_state =
                                        GuiStateMachine::<TargetSelect>::transition_from(
                                            &next_gui_state,
                                            TargetSelectArgs {
                                                card_idx,
                                                targets: vec![enemy],
                                            },
                                        );
                                    self.gui_state = GuiState::TargetSelect(next_gui_state);
                                }
                            }
                        }
                    }
                    _ => ()
                }
            }
            GuiState::TargetSelect(ref mut state) => {
                match input {
                    Key::Char('q') => {
                        // Cancel by resetting back to initial GUI
                        // state
                        let next_gui_state =
                            GuiStateMachine::<Combat>::new(self.game_state.enemy.unwrap());
                        self.gui_state = GuiState::Combat(next_gui_state);
                    }
                    Key::Char('1') => {
                        // Transition back to Combat state and
                        // play the card now that the player
                        // selected a target
                        let entity_idx = 1;
                        let target = self.game_state.entities[entity_idx];
                        let next_gui_state =
                            GuiStateMachine::<TargetSelectComplete>::transition_from(
                                state,
                                TargetSelectCompleteArgs { target },
                            );
                        self.gui_state = GuiState::TargetSelectComplete(next_gui_state);
                    }
                    _ => {}
                }
            }
            // TODO this shouldn't be here since it's not
            // actually handling any user input, just handling the
            // state machine
            GuiState::TargetSelectComplete(ref state) => {
                // Reset to combat state
                // TODO maybe make this an explicit transition?
                let target_id = state.state.target;
                let card_idx = state.state.card_idx;

                let next_gui_state = GuiStateMachine::<Combat>::new(self.game_state.enemy.unwrap());
                self.gui_state = GuiState::Combat(next_gui_state);

                // Set the action to be processed next tick
                self.game_state.action = Action::PlayCard(target_id, card_idx as i32);
            }
        }

        self
    }

    fn update(&mut self) -> &mut Self {
        match self.gui_state {
            GuiState::TargetSelectComplete(ref state) => {
                // Reset to combat state
                // TODO maybe make this an explicit transition?
                let target_id = state.state.target;
                let card_idx = state.state.card_idx;

                let next_gui_state = GuiStateMachine::<Combat>::new(self.game_state.enemy.unwrap());
                self.gui_state = GuiState::Combat(next_gui_state);

                // Set the action to be processed next tick
                self.game_state.action = Action::PlayCard(target_id, card_idx as i32);
            }
            _ => ()
        }
        // Move the game forward one tick
        tick(&mut self.game_state);
        // Await user input
        self.game_state.action = Action::Await;

        self
    }
}

/// Shows a backtrace if the program panics
fn panic_hook(info: &PanicInfo<'_>) {
    if cfg!(debug_assertions) {
        let location = info.location().unwrap();

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let stacktrace: String = format!("{:?}", Backtrace::new()).replace('\n', "\n\r");

        println!(
            "{}thread '<unnamed>' panicked at '{}', {}\n\r{}",
            termion::screen::ToMainScreen,
            msg,
            location,
            stacktrace
        );
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    // Initialize the terminal
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup event handlers
    let config = Config {
        tick_rate: Duration::from_millis(250),
        ..Default::default()
    };
    let events = Events::with_config(config);

    // Initialize the game
    let mut game = Game::new();

    loop {
        terminal.draw(|f| {
            let game_state = &game.game_state;

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(40),
                        Constraint::Percentage(40),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // Display the player's status

            let player_state = game_state
                .entity_state
                .get(&game_state.player)
                .expect("Failed to get player's state")
                .get_state();

            // Use deref coercion to convert to &str. Using just &
            // operator, the compiler will automatically insert an
            // appropriate amount of derefs (*) based on the context
            let player_status: &str = &format!(
                "Shields: {}  /  Hull: {}",
                player_state.get(&Attribute::Shields).unwrap(),
                player_state.get(&Attribute::Hull).unwrap(),
            );

            let status_bar = Paragraph::new(player_status)
                .block(Block::default().borders(Borders::ALL).title("Status"))
                .alignment(Alignment::Center);

            f.render_widget(status_bar, chunks[0]);

            // Display the enemy

            let enemy_state = game_state
                .entity_state
                .get(&game_state.enemy.unwrap())
                .expect("Failed to get enemy's state")
                .get_state();

            let enemy_status: &str = &format!(
                "Shields: {}  /  Hull: {}",
                enemy_state.get(&Attribute::Shields).unwrap(),
                enemy_state.get(&Attribute::Hull).unwrap(),
            );

            let mut text: Vec<Spans> = SPACE_SHIP.split('\n').map(|l| Spans::from(l)).collect();
            text.push(Spans::from(""));
            text.push(Spans::from(enemy_status));

            let paragraph = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::LightYellow))
                .alignment(Alignment::Left);

            f.render_widget(paragraph, chunks[1]);

            // Show the deck piles (draw pile, hand, discard pile)

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(25),
                        Constraint::Percentage(50),
                        Constraint::Percentage(25),
                    ]
                    .as_ref(),
                )
                .split(chunks[2]);

            let draw_pile = Block::default()
                .title("List")
                .borders(Borders::ALL)
                .title("Draw");

            f.render_widget(draw_pile, horizontal_chunks[0]);

            let items: Vec<ListItem> = game_state
                .hand
                .iter()
                .map(|i| ListItem::new(game_state.cards.get(i).unwrap().name))
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Hand"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");

            f.render_widget(list, horizontal_chunks[1]);

            let discard_items = vec![];

            let discard_pile = List::new(discard_items)
                .block(Block::default().borders(Borders::ALL).title("Discard"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");

            f.render_widget(discard_pile, horizontal_chunks[2]);

            // Show the player input prompt

            // Accumulate the list of cards in the hand with a number
            // to press to play it
            let mut cards_to_play = String::new();
            for (idx, i) in game_state.hand.iter().enumerate() {
                let name = game_state.cards.get(i).unwrap().name;
                cards_to_play.push_str(&format!("[{}]{} ", idx + 1, name));
            }

            let prompt = Paragraph::new(vec![
                Spans::from("Select a card to play"),
                Spans::from(Span::styled(
                    cards_to_play,
                    Style::default().fg(Color::LightGreen),
                )),
            ])
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

            f.render_widget(prompt, chunks[3]);

            if let GuiState::TargetSelect(state) = &game.gui_state {
                // Create a centered modal
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(33),
                            Constraint::Percentage(33),
                            Constraint::Percentage(33),
                        ]
                            .as_ref(),
                    )
                    .split(f.size());

                let horizontal_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(20),
                            Constraint::Percentage(60),
                            Constraint::Percentage(20),
                        ]
                            .as_ref(),
                    )
                    .split(chunks[1]);
                let modal = horizontal_chunks[1];

                // Clear it so the background is blank
                f.render_widget(Clear, modal);

                let mut targets = String::new();
                for (idx, i) in state.state.targets.iter().enumerate() {
                    let name = &*game_state.entity_state.get(i).unwrap().get_name();
                    targets.push_str(&format!("[{}]{} ", idx + 1, name));
                }

                let prompt = Paragraph::new(vec![
                    Spans::from("Select a target"),
                    Spans::from(Span::styled(
                        targets,
                        Style::default().fg(Color::LightGreen),
                    )),
                ])
                    .block(Block::default()
                           .borders(Borders::ALL)
                           .style(Style::default().bg(Color::Black)))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false });

                f.render_widget(prompt, modal);
            }
        })?;

        match events.next()? {
            Event::Tick => game.update(),
            Event::Input(Key::Char('q')) => {
                break;
            },
            Event::Input(input) => game.handle_keyboard_input(input),
        };
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    panic::set_hook(Box::new(|info| {
        panic_hook(info);
    }));

    run()
}
