//! Terminal UI for potions game
//!
//! Originally derived from Ratatui canvas example.

use std::{
    io::{self, stdout, Stdout, Write, Read},
    time::{Duration, Instant},
    fs::{self, File},
    env,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{layout::Flex, prelude::*, widgets::*};

use potions::vial_physics::VialPhysics;
use potions::*;

fn usage() -> io::Result<()> {
    eprintln!("Usage: termui");
    eprintln!("       termui write <dir>");
    eprintln!("       termui read <dir>");
    Ok(())
}

fn main() -> io::Result<()> {
    let mut app = App::new();
    let mut args = env::args();
    let _ = args.next();
    match args.next().as_deref() {
        Some("write") => {
            write_levels(&args.next().expect("dir"), &app.levels)
        },
        Some("read") => {
            app.levels = read_levels(&args.next().expect("dir"))?;
            app.run()
        }
        Some(command) => {
            eprintln!("error: invalid command {:?}.", command);
            usage()
        }
        None => {
            app.run()
        }
    }
}

fn write_levels(dir: &str, levels: &[Level]) -> io::Result<()> {
    for (i, level) in levels.iter().enumerate() {
        let mut file = File::create(format!("{dir}/{}.ron", i))?;
        file.write_all(ron::ser::to_string_pretty(&level,
         ron::ser::PrettyConfig::default()).unwrap().as_bytes())?;
        // let mut file = File::create(format!("levels/{}.toml", i)).expect("file create");
        // file.write_all(toml::ser::to_string_pretty(&level).unwrap().as_bytes()).expect("write");
    }
    Ok(())
}

fn read_levels(dir: &str) -> io::Result<Vec<Level>> {
    let mut levels = vec![];
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "ron" {
                    let mut file = File::open(&path)?;
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    levels.push(ron::from_str(&contents).expect("level"));
                }
            }
        }
    }
    Ok(levels)
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
    Transfer(Transfer, f32),
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
        if index >= self.levels.len() {
            false
        } else {
            self.potions = self.levels[index].potions.to_vec();
            self.vial_physics = self.potions.iter().map(VialPhysics::new).collect();
            self.level_index = index;
            true
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut terminal = init_terminal()?;
        // let mut self = Self::new();
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(16);
        // panic::catch_unwind(|| {
        //     let _ = restore_terminal();
        // }).unwrap();
        loop {
            let _ = terminal.draw(|frame| self.ui(frame));
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => break,
                        _ => {}
                    }
                    match self.state {
                        State::NextLevel => {
                            if self.goto_level(self.level_index + 1) {
                                self.state = State::Game;
                            } else {
                                self.state = State::End;
                            }
                        }
                        State::Game => match key.code {
                            KeyCode::Char('m') => {
                                let palette = &mut self.levels[self.level_index].palette;
                                self.potions[self.cursor].mix(palette);
                            }
                            KeyCode::Char('n') => {
                                self.goto_level(self.level_index + 1);
                                // self.step(Duration::from_secs_f32(1.0 / 60.0));
                            }
                            KeyCode::Char('p') => {
                                self.goto_level(self.level_index.saturating_sub(1));
                            }
                            KeyCode::Char(' ') | KeyCode::Up => match self.selected {
                                Some(i) => {
                                    if i == self.cursor {
                                        self.selected = None;
                                    } else if let Some(transfer) =
                                        self.potions[i].pour(&self.potions[self.cursor])
                                    {
                                        self.state = State::Transfer(transfer, 0.0);
                                    } else {
                                        self.selected = None;
                                    }
                                }
                                None => self.selected = Some(self.cursor),
                            },
                            KeyCode::Right | KeyCode::Char('l') => {
                                self.cursor = (self.cursor + 1).rem_euclid(self.potions.len())
                            }
                            KeyCode::Left | KeyCode::Char('h') => {
                                self.cursor = (self.cursor + self.potions.len() - 1)
                                    .rem_euclid(self.potions.len())
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
                self.on_tick(current - last_tick);
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
