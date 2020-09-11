use std::{error::Error, io, time::Duration};
use std::panic::{self, PanicInfo};
use backtrace::Backtrace;

use termion::{
    event::Key,
    input::MouseTerminal,
    raw::IntoRawMode,
    screen::AlternateScreen,
};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Modifier, Style, Color},
    text::{Spans, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Wrap
    },
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
}

struct Game {
    game_state: GameState,
    gui_state: GuiState,
}

impl Game {
    fn init_state() -> GameState {
        let mut cards = CardCollection::new();

        cards.insert(
            Card {
                id: CardId::Shields,
                name: "Shields",
                effects: vec![Box::new(IncreaseShields {})],
                target: Target::Player
            }
        );

        cards.insert(
            Card {
                id: CardId::Phasers,
                name: "Phasers",
                effects: vec![Box::new(DamageHull {})],
                target: Target::Single
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

        let mut game_state = GameState::new(cards, init_deck);

        // Add player
        let mut s = State::new();
        s.insert(Attribute::Hull, 10);
        s.insert(Attribute::Shields, 10);
        let player = Player { state: s };
        let player_id = 1;
        game_state.add_entity(Some(player_id), Box::new(player));
        game_state.player = player_id;

        // Add an enemy
        let mut s = State::new();
        s.insert(Attribute::Hull, 10);
        s.insert(Attribute::Shields, 10);
        let enemy = Enemy { state: s };
        let enemy_id = 2;
        game_state.add_entity(Some(enemy_id), Box::new(enemy));
        game_state.enemy = Some(enemy_id);

        draw_hand(&mut game_state, 4);

        game_state
    }

    fn new() -> Self {
        let game_state = Self::init_state();
        let gui_state = GuiState::Combat(GuiStateMachine::<Combat>::new(
            None,
            game_state.enemy.unwrap(),
        ));

        Self {
            game_state,
            gui_state
        }
    }

    fn update(&mut self) {
        // Move the game forward one tick
        tick(&mut self.game_state);
        // Await user input
        self.game_state.action = Action::Await;
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
                .constraints([Constraint::Percentage(10),
                              Constraint::Percentage(40),
                              Constraint::Percentage(40),
                              Constraint::Percentage(10)].as_ref())
                .split(f.size());

            // Display the player's status

            let player_state = game_state.entity_state.get(&game_state.player)
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

            let enemy_state = game_state.entity_state.get(&game_state.enemy.unwrap())
                .expect("Failed to get enemy's state")
                .get_state();

            let enemy_status: &str = &format!(
                "Shields: {}  /  Hull: {}",
                enemy_state.get(&Attribute::Shields).unwrap(),
                enemy_state.get(&Attribute::Hull).unwrap(),
            );

            let mut text: Vec<Spans> = SPACE_SHIP.split('\n')
                .map(|l| Spans::from(l))
                .collect();
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
                .constraints([Constraint::Percentage(25),
                              Constraint::Percentage(50),
                              Constraint::Percentage(25)].as_ref())
                .split(chunks[2]);

            let draw_pile = Block::default().title("List").borders(Borders::ALL).title("Draw");
            f.render_widget(draw_pile, horizontal_chunks[0]);

            let items: Vec<ListItem> = game_state.hand.iter()
                .map(|i| ListItem::new(game_state.cards.get(i).unwrap().name))
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Hand"))
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");

            f.render_widget(list, horizontal_chunks[1]);

            let discard_items = vec![];

            let discard_pile = List::new(discard_items)
                .block(Block::default().borders(Borders::ALL).title("Discard"))
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default().add_modifier(Modifier::ITALIC))
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

            let prompt = Paragraph::new(
                vec![
                    Spans::from("Select a card to play"),
                    Spans::from(Span::styled(cards_to_play, Style::default().fg(Color::LightGreen)))
                ])
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false });

            f.render_widget(prompt, chunks[3]);

            if let GuiState::TargetSelect(_state) = &game.gui_state {
                // TODO if in a targeting state, show modal
            }
        })?;

        // Input events are handled differently depending on the UI
        // state machine
        match game.gui_state {
            GuiState::Combat(ref state) => {
                match events.next()? {
                    Event::Input(input) => match input {
                        Key::Char('q') => {
                            break;
                        }
                        Key::Char('e') => {
                            game.game_state.action = Action::EndTurn;
                        }
                        Key::Char(num_char) => {
                            if ['1', '2', '3', '4', '5', '6', '7', '8', '9'].contains(&num_char) && num_char.to_digit(10).unwrap() <= game.game_state.hand.len() as u32 {
                                let card_idx = num_char.to_digit(10).unwrap() as usize;
                                let card_idx = card_idx - 1; // Convert to vector index
                                let card_id = game.game_state.hand[card_idx];
                                let selected_card = game.game_state.cards.get(&card_id).unwrap();

                                // Determine the target of the card or
                                // prompt the user
                                match selected_card.target {
                                    Target::Player => {
                                        game.game_state.action = Action::PlayCard(game.game_state.player, card_idx as i32);
                                    }
                                    Target::Single => {
                                        // TODO If there is only a single
                                        // enemy then skip the transition
                                        let mut next_gui_state = GuiStateMachine::<TargetSelect>::from(state);
                                        next_gui_state.state.shared_state.card_idx = Some(card_idx as i32);
                                        game.gui_state = GuiState::TargetSelect(next_gui_state);
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    Event::Tick => {
                        game.update();
                    }
                }
            },
            GuiState::TargetSelect(ref mut state) => {
                match events.next()? {
                    Event::Input(input) => match input {
                        Key::Char('q') => {
                            // TODO cancel by moving back to previous
                            // GUI state
                            // break;
                        }
                        Key::Char('1') => {
                            // Transition back to Combat state and
                            // play the card now that the player
                            // selected a target
                            let entity_idx = 1;
                            let target_id = game.game_state.entities[entity_idx];

                            // TODO this feels wrong, probably an
                            // indication there should actually be
                            // another state to transition to and
                            // target_id should not be in shared state
                            // state.state.shared_state.target_id = Some(target_id);
                            let card_idx = state.state.shared_state.card_idx.unwrap();

                            let next_gui_state = GuiStateMachine::<Combat>::from(state);
                            game.gui_state = GuiState::Combat(next_gui_state);
                            game.game_state.action = Action::PlayCard(target_id, card_idx);
                        }
                        _ => {}
                    },
                    Event::Tick => {
                        game.update();
                    }
                }
            },
        }


    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    panic::set_hook(Box::new(|info| {
        panic_hook(info);
    }));

    run()
}
