use std::{error::Error, io, time::Duration};
use termion::{
    event::Key,
    input::MouseTerminal,
    raw::IntoRawMode,
    screen::AlternateScreen
};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Color},
    widgets::{
        canvas::{Canvas, Map, MapResolution, Rectangle},
        Block, Borders, List, ListItem
    },
    Terminal,
};

mod engine;
mod event;

use engine::*;
use event::{Config, Event, Events};

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

        // Add an enemy
        let mut s = State::new();
        s.insert(Attribute::Hull, 10);
        s.insert(Attribute::Shields, 10);
        let enemy = Enemy { state: s };
        let enemy_id = game_state.add_entity(Box::new(enemy));

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
        // TODO
    }
}

fn main() -> Result<(), Box<dyn Error>> {
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
                .constraints([Constraint::Percentage(50),
                              Constraint::Percentage(50)].as_ref())
                .split(f.size());

            let items: Vec<ListItem> = game_state.hand.iter()
                .map(|i| ListItem::new(game_state.cards.get(i).unwrap().name))
                .collect();

            let list = List::new(items)
                .block(Block::default().title("List").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");

            f.render_widget(list, chunks[0]);

            let canvas = Canvas::default()
                .block(Block::default().borders(Borders::ALL).title("Tunnelcast"))
                .paint(|ctx| {
                    ctx.print(game.x, -game.y, "Cards go here", Color::Green);
                })
                .x_bounds([-180.0, 180.0])
                .y_bounds([-90.0, 90.0]);
            f.render_widget(canvas, chunks[0]);
            let canvas = Canvas::default()
                .block(Block::default().borders(Borders::ALL))
                .paint(|ctx| {
                    ctx.print(game.x, -game.y, "You are here", Color::Yellow);
                })
                .x_bounds([10.0, 110.0])
                .y_bounds([10.0, 110.0]);
            f.render_widget(canvas, chunks[1]);
        })?;

        match events.next()? {
            Event::Input(input) => match input {
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
