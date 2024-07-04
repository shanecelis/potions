//! Terminal UI for potions game
//!
//! Originally derived from Ratatui canvas example.
use bevy_ecs::prelude::*;
use bevy_app::AppExit;
use std::{
    env,
    fs::{self, File},
    io::{self, stdout, Read, Stdout, Write},
    time::{Duration},
    collections::HashMap,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use derived_deref::{Deref, DerefMut};
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
    let mut app = bevy_app::App::new();
    let mut my_app = App::new();
    let mut args = env::args();
    let _ = args.next();
    match args.next().as_deref() {
        Some("write") => {
            return write_levels(&args.next().expect("dir"), &my_app.levels);
        }
        Some("read") => {
            my_app.levels = read_levels(&args.next().expect("dir"))?;
        }
        Some(command) => {
            eprintln!("error: invalid command {:?}.", command);
            return usage();
        }
        None => ()
    }
    let terminal = init_terminal()?;
    app.insert_resource(my_app);
    app.insert_resource(Term(terminal));
    app.add_systems(bevy_app::Update, update_app);
    app.add_systems(bevy_app::Last, check_exit);
    let mut cont = true;
    while cont {
        app.update();
        let my_app = app.world.resource::<App>();
        cont = !matches!(my_app.state, State::End);
    }
    restore_terminal()
}

fn update_app(mut my_app: ResMut<App>,
              mut terminal: ResMut<Term>,
              mut app_exit_events: ResMut<Events<bevy_app::AppExit>>
) {
    let r = my_app.update(&mut terminal);
    if !matches!(r, Ok(true)) {
        app_exit_events.send(bevy_app::AppExit);
    }
}

fn check_exit(mut app_exit: EventReader<AppExit>, mut my_app: ResMut<App>) {
    if app_exit.read().next().is_some() {
        my_app.state = State::End;
    }
}

fn write_levels(dir: &str, levels: &[Level]) -> io::Result<()> {
    for (i, level) in levels.iter().enumerate() {
        let mut file = File::create(format!("{dir}/{}.ron", i))?;
        file.write_all(
            ron::ser::to_string_pretty(&level, ron::ser::PrettyConfig::default())
                .unwrap()
                .as_bytes(),
        )?;
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
#[derive(Resource)]
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

#[derive(Resource)]
struct ScriptChannels {
    input: Sender<Input>,
    output: Receiver<Output>
}

#[derive(Resource, Deref, DerefMut)]
struct Term(Terminal<CrosstermBackend<Stdout>>);

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
        let mut my_app = App {
            tick_count: 0,
            cursor: 0,
            selected: None,
            level_index: 0,
            potions: vec![],
            vial_physics: vec![],
            levels,
            state: State::Game,
        };
        my_app.goto_level(my_app.level_index);
        my_app
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

    pub fn update(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<bool> {
        let _ = terminal.draw(|frame| self.ui(frame));

        let timeout = Duration::from_millis(16);
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    let quit = match key.code {
                        KeyCode::Char('q') => true,
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => true,
                        _ => false
                    };
                    if quit {
                        return Ok(false);
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
                            KeyCode::Char('r') => {
                                self.goto_level(self.level_index);
                            }
                            KeyCode::Char('n') => {
                                self.goto_level(self.level_index + 1);
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

            // if last_tick.elapsed() >= tick_rate {
            //     let current = Instant::now();
                self.on_tick();
            //     last_tick = current;
            // }
        // }
        Ok(true)
    }

    fn sync_objects(&mut self, vial_index: usize) {
        for obj in &self.potions[vial_index].objects {
            self.vial_physics[vial_index].insert(obj);
        }
    }

    fn step(&mut self) {
        for (i, potion) in self.potions.iter_mut().enumerate() {
            let phys = &mut self.vial_physics[i];
            phys.kick_on_enter(potion);
            phys.add_buoyancy_forces(potion);
            phys.step();

            let mut map: HashMap<u128, &mut Object> =
                potion.objects.iter_mut().map(|o| (o.id as u128, o)).collect();
            phys.handle_collisions(&mut map).expect("collision");
            phys.project(potion);
        }
    }

    fn on_tick(&mut self) {
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
                    if let Some(Transition::BreakSeed(vial) | Transition::MoveDown(vial)) = potion.transition() {
                        sync.push(i);
                        *potion = vial;
                    }
                }
            }
            State::NextLevel => (),
            State::End => (),
        }
        for i in &sync {
            self.sync_objects(*i);
        }
        if matches!(self.state, State::Game) {
            self.step();
            if ! sync.is_empty() {
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
