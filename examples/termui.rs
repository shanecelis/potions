//! Terminal UI for potions game
//!
//! Originally derived from Ratatui canvas example.

use std::{
    io::{self, stdout, Stdout},
    time::{Duration, Instant},
    panic,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{layout::Flex, prelude::*, widgets::*};

use potions::*;
use potions::vial_physics::VialPhysics;

fn main() -> io::Result<()> {
    App::run()
}

struct App {
    tick_count: u64,
    potions: Vec<Vial>,
    vial_physics: Vec<VialPhysics>,
    cursor: usize,
    selected: Option<usize>,
    levels: Vec<Level>,
    level_index: usize,
    state: State,
}

#[derive(Debug)]
enum State {
    Game,
    NextLevel,
    Transfer(Transfer, f64),
    // Pouring(Vial, Vial, f64),
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
        let mut app = App {
            tick_count: 0,
            cursor: 0,
            selected: None,
            level_index: 0,
            potions: vec![],
            vial_physics: vec![],
            levels,
            state: State::Game,
        };
        app.goto_level(app.level_index);
        app
    }

    pub fn goto_level(&mut self, index: usize) -> bool {
        if index >= self.levels.len() || index < 0 {
            false
        } else {
            self.potions = self.levels[index]
                .potions
                .iter()
                .cloned()
                .collect();
            self.vial_physics = self.potions.iter().map(VialPhysics::new).collect();
            self.level_index = index;
            true
        }
    }

    pub fn run() -> io::Result<()> {
        let mut terminal = init_terminal()?;
        let mut app = App::new();
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(16);
        // panic::catch_unwind(|| {
        //     let _ = restore_terminal();
        // }).unwrap();
        loop {
            let _ = terminal.draw(|frame| app.ui(frame));
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => break,
                        _ => {}
                    }
                    match app.state {
                        State::NextLevel => {
                            if app.goto_level(app.level_index + 1) {
                                app.state = State::Game;
                            } else {
                                app.state = State::End;
                            }
                        }
                        State::Game => match key.code {
                            KeyCode::Char('m') => {
                                let palette = &mut app.levels[app.level_index].palette;
                                app.potions[app.cursor].mix(palette);
                            },
                            KeyCode::Char('n') => {
                                app.goto_level(app.level_index + 1);
                                // app.step(Duration::from_secs_f32(1.0 / 60.0));
                            },
                            KeyCode::Char('p') => {
                                app.goto_level(app.level_index.saturating_sub(1));
                            },
                            KeyCode::Char(' ') | KeyCode::Up => match app.selected {
                                Some(i) => {
                                    if i == app.cursor {
                                        app.selected = None;
                                    } else {
                                        if let Some(transfer) =
                                            app.potions[i].pour(&app.potions[app.cursor])
                                        {
                                            app.state = State::Transfer(transfer, 0.0);
                                        } else {
                                            app.selected = None;

                                        }
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
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                let current = Instant::now();
                app.on_tick(current - last_tick);
                last_tick = current;
            }
        }
        restore_terminal()?;
        Ok(())
    }

    fn sync_objects(&mut self, vial_index: usize) {
        for obj in &self.potions[vial_index].objects {
            self.vial_physics[vial_index].insert(obj);
        }
    }

    fn step(&mut self, delta: Duration) {
        for (i, potion) in self.potions.iter_mut().enumerate() {
            let phys = &mut self.vial_physics[i];
            phys.add_buoyancy_forces(potion);
            phys.step(delta.as_secs_f32());
            phys.project(potion);
        }
    }

    fn on_tick(&mut self, delta: Duration) {
        self.tick_count += 1;
        let mut sync = vec![];
        match self.state {
            State::Transfer(ref transfer, ref mut t) => {
                let i = self.selected.unwrap();
                let j = self.cursor;
                let pour_from = &self.potions[i];
                let pour_into = &self.potions[j];
                if let Some((a, b)) = transfer.lerp(pour_from, pour_into, *t) {
                    self.potions[i] = a;
                    self.potions[j] = b;
                    sync.push(i);
                    sync.push(j);
                } else {
                    *t = 2.0;
                }
                *t += 0.1;
                if *t >= 1.0 {
                    self.selected = None;
                    if self.levels[self.level_index]
                        .goal
                        .is_complete(&self.potions)
                    {
                        self.state = State::NextLevel;
                    } else {
                        self.state = State::Game;
                    }
                }
            }
            State::Game => {
                for (i, potion) in self.potions.iter_mut().enumerate() {
                    match potion.transition() {
                        Some(Transition::BreakSeed(vial) | Transition::MoveDown(vial)) => {
                            sync.push(i);
                            *potion = vial;
                        }
                        None => (),
                    }
                }
            }
            State::NextLevel => (),
            State::End => (),
            _ => todo!("{:?}", self.state),
        }
        for i in sync {
            self.sync_objects(i);
        }
        if matches!(self.state, State::Game) {
            self.step(delta);
        }
    }

    fn ui(&self, frame: &mut Frame) {
        match self.state {
            State::Game | State::Transfer(_, _) => self.render_game(frame),
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
            _ => todo!(),
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
        let palette = &self.levels[self.level_index].palette;

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
