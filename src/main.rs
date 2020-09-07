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

use engine::*;
use event::{Config, Event, Events};


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

struct Game {
    game_state: GameState,
    x: f64,
    y: f64,
}

impl Game {
    fn init_state() -> GameState {
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

        let mut game_state = GameState::new(cards, init_deck);

        // Add player
        let mut s = State::new();
        s.insert(Attribute::Hull, 10);
        s.insert(Attribute::Shields, 10);
        let player = Player { state: s };
        game_state.add_entity(Box::new(player));

        // Add an enemy
        let mut s = State::new();
        s.insert(Attribute::Hull, 10);
        s.insert(Attribute::Shields, 10);
        let enemy = Enemy { state: s };
        game_state.add_entity(Box::new(enemy));

        draw_hand(&mut game_state, 4);

        game_state
    }

    fn new() -> Self {
        let game_state = Self::init_state();
        Self {
            game_state,
            x: 0.0,
            y: 0.0,
        }
    }

    fn update(&mut self) {
        tick(&mut self.game_state);
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

            let status_bar = Paragraph::new("Shields: 33  /  Hull: 33")
                .block(Block::default().borders(Borders::ALL).title("Status"))
                .alignment(Alignment::Center);
            f.render_widget(status_bar, chunks[0]);

            let mut text: Vec<Spans> = SPACE_SHIP.split('\n')
                .map(|l| Spans::from(l))
                .collect();
            text.push(Spans::from(""));
            text.push(Spans::from("Shields: 10  /  Hull: 10"));

            let paragraph = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::LightYellow))
                .alignment(Alignment::Left);
            f.render_widget(paragraph, chunks[1]);

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

            // Accumulate the list of cards in the hand with a number
            // to press to play it
            let mut cards_to_play = String::new();
            for (idx, i) in game_state.hand.iter().enumerate() {
                let name = game_state.cards.get(i).unwrap().name;
                cards_to_play.push_str(&format!("[{}]{} ", idx, name));
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
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('1') => {
                    let card_id = game.game_state.hand[1];
                    let _selected_card = game.game_state.cards.get(&card_id).unwrap();
                    // TODO Prompt for who the target is
                    let target_entity = game.game_state.entities[1];
                    game.game_state.action = Action::PlayCard(target_entity, 1);
                }
                Key::Char('q') => {
                    break;
                }
                Key::Down => {
                    game.y += 1.0;
                }
                Key::Up => {
                    game.y -= 1.0;
                }
                Key::Right => {
                    game.x += 1.0;
                }
                Key::Left => {
                    game.x -= 1.0;
                }

                _ => {}
            },
            Event::Tick => {
                game.update();
            }
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
