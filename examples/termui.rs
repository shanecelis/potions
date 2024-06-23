//! Terminal UI for potions game
//!
//! Originally derived from Ratatui canvas example.

use std::{
    io::{self, stdout, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{layout::Flex, prelude::*, widgets::*};

use potions::*;

fn main() -> io::Result<()> {
    App::run()
}

// #[derive(Debug)]
struct App {
    tick_count: u64,
    potions: Vec<Vial>,
    cursor: usize,
    selected: Option<usize>,
    // #[skip]
    levels: Vec<Box<dyn Level>>,
    level_index: usize,
    state: State,
}

#[derive(Debug)]
enum State {
    Game,
    NextLevel,
    Pouring(Vial, Vial, f64),
    End,
}

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl App {
    fn new() -> App {
        let levels = levels();
        App {
            tick_count: 0,
            cursor: 0,
            selected: None,
            level_index: 0,
            potions: levels[0].potions().iter().cloned().collect(),
            levels,
            state: State::Game,
        }
    }

    pub fn run() -> io::Result<()> {
        let mut terminal = init_terminal()?;
        let mut app = App::new();
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(16);
        loop {
            let _ = terminal.draw(|frame| app.ui(frame));
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        _ => {}
                    }
                    match app.state {
                        State::NextLevel => {
                            if app.level_index + 1 >= app.levels.len() {
                                app.state = State::End;
                            } else {
                                app.level_index += 1;
                                app.potions = app.levels[app.level_index]
                                    .potions()
                                    .into_iter()
                                    .cloned()
                                    .collect();
                                app.state = State::Game;
                            }
                        }
                        State::Game => match key.code {
                            KeyCode::Char(' ') => match app.selected {
                                Some(i) => {
                                    if i == app.cursor {
                                        app.selected = None;
                                    } else {
                                        app.state = State::Pouring(
                                            app.potions[i].clone(),
                                            app.potions[app.cursor].clone(),
                                            0.0,
                                        );
                                    }
                                }
                                None => app.selected = Some(app.cursor),
                            },
                            KeyCode::Right | KeyCode::Char('l') => {
                                app.cursor = (app.cursor + 1).rem_euclid(app.potions.len())
                            }
                            KeyCode::Left | KeyCode::Char('h') => {
                                app.cursor = (app.cursor + app.potions.len() - 1)
                                    .rem_euclid(app.potions.len())
                            }
                            _ => {}
                        },
                        State::End => {}
                            // match key.code {
                            //     _ => break
                            // }
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                app.on_tick();
                last_tick = Instant::now();
            }
        }
        restore_terminal()?;
        Ok(())
    }

    fn on_tick(&mut self) {
        self.tick_count += 1;
        match self.state {
            State::Pouring(ref pour_from, ref pour_into, ref mut t) => {
                if let Some((a, b)) = pour_from.pour(pour_into, *t) {
                    self.potions[self.selected.unwrap()] = a;
                    self.potions[self.cursor] = b;
                }
                *t += 0.1;
                if *t >= 1.0 {
                    self.selected = None;
                    if self.levels[self.level_index].is_complete(&self.potions) {
                        self.state = State::NextLevel;
                    } else {
                        self.state = State::Game;
                    }
                }
            }
            _ => (),
        }
    }

    fn ui(&self, frame: &mut Frame) {
        match self.state {
            State::Game | State::Pouring(_, _, _) => self.render_game(frame),
            State::NextLevel => {
                self.render_game(frame);
                let rect = centered_rect(frame.size(), 35, 35);
                frame.render_widget(Clear, rect);
                frame.render_widget(
                    Paragraph::new(format!("You passed level {}", self.level_index + 1))
                        .block(Block::default().borders(Borders::all()))
                        .alignment(Alignment::Center),
                    rect,
                )
            }
            State::End => {
                self.render_game(frame);
                let rect = centered_rect(frame.size(), 35, 35);
                frame.render_widget(Clear, rect);
                frame.render_widget(
                    Paragraph::new("You finished the game!\nThanks for playing.\nHit 'q' to quit.")
                        .block(Block::default().borders(Borders::all()))
                        .alignment(Alignment::Center),
                    rect,
                )
            }
        }
    }

    fn render_game(&self, frame: &mut Frame) {
        let [title, content] =
            Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)])
                .areas(frame.size());
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(std::iter::repeat(Constraint::Fill(1)).take(self.potions.len()))
            .margin(5)
            .flex(Flex::Center)
            .spacing(10);
        frame.render_widget(
            Paragraph::new(format!("Level {}", self.level_index + 1)).alignment(Alignment::Center),
            title,
        );
        let palette = self.levels[self.level_index].palette();

        for (i, rect) in layout.split(content).iter().enumerate() {
            let selected = self.selected.map(|x| x == i).unwrap_or(false);

            let [_gap, potion, footer] = Layout::vertical([
                Constraint::Min(if selected { 0 } else { 1 }),
                Constraint::Percentage(100),
                Constraint::Min(if selected { 2 } else { 1 }),
            ])
            .areas(*rect);

            frame.render_widget(tui::VialWidget(&self.potions[i], palette), potion);

            frame.render_widget(
                Paragraph::new(if self.cursor == i { "/\\" } else { "" })
                    .alignment(Alignment::Center),
                footer,
            );
        }
    }
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
