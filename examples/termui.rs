//! # [Ratatui] Canvas example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui-org/ratatui
//! [examples]: https://github.com/ratatui-org/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui-org/ratatui/blob/main/examples/README.md

use std::{
    io::{self, stdout, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{canvas::*, *},
    layout::Flex,
};

use color_art;
use potions::*;

fn main() -> io::Result<()> {
    App::run()
}

// let cursor: usize = 0;
// let count: usize = 5;
// // Move cursor to the right.
// // cursor + 1 % count
// KeyCode::Right => (cursor + 1).rem_euclid(count),
// // Move cursor backwards. cursor - 1 % count
// KeyCode::Left => (cursor + count - 1).rem_euclid(count),

struct App {
    x: f64,
    y: f64,
    ball: Circle,
    playground: Rect,
    vx: f64,
    vy: f64,
    tick_count: u64,
    marker: Marker,
    potions: Vec<Potion>,
    cursor: usize,
    selected: Option<usize>
}

impl App {
    fn new() -> App {
        App {
            x: 0.0,
            y: 0.0,
            ball: Circle {
                x: 20.0,
                y: 40.0,
                radius: 10.0,
                color: Color::Yellow,
            },
            playground: Rect::new(10, 10, 200, 100),
            vx: 1.0,
            vy: 1.0,
            tick_count: 0,
            marker: Marker::Braille,
            cursor: 0,
            selected: None,
            potions: vec![
                Potion {
                    layers: vec![
                        Layer::Liquid { color:
                                        color_art::Color::from_rgb(255, 0, 0).unwrap(),
                                        volume: 50.0 },

                        Layer::Liquid { color:
                                        color_art::Color::from_rgb(0, 255, 0).unwrap(),
                                        volume: 25.0 },
                    ],
                    ..Default::default()
                },

                Potion {
                    layers: vec![
                        Layer::Liquid { color:
                                        color_art::Color::from_rgb(0, 255, 0).unwrap(),
                                        volume: 50.0 },

                        Layer::Liquid { color:
                                        color_art::Color::from_rgb(0, 0, 255).unwrap(),
                                        volume: 25.0 },
                    ],
                    ..Default::default()
                },

                Potion {
                    layers: vec![
                        Layer::Liquid { color:
                                        color_art::Color::from_rgb(0, 0, 255).unwrap(),
                                        volume: 50.0 } ],
                    ..Default::default()
                },
            ]
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
                        KeyCode::Char(' ') => {
                            match app.selected {
                                Some(i) => {
                                    if i == app.cursor {
                                        app.selected = None;
                                    } else {
                                        let pour_from = &app.potions[i];
                                        let pour_into = &app.potions[app.cursor];
                                        if let Some((a, b)) = pour_from.pour(pour_into) {
                                            app.potions[i] = a;
                                            app.potions[app.cursor] = b;
                                        }
                                        app.selected = None;
                                    }
                                },
                                None => app.selected = Some(app.cursor)
                            }
                        },
                        KeyCode::Down | KeyCode::Char('j') => app.y += 1.0,
                        KeyCode::Up | KeyCode::Char('k') => app.y -= 1.0,
                        KeyCode::Right | KeyCode::Char('l') => app.cursor = (app.cursor + 1).rem_euclid(app.potions.len()),
                        KeyCode::Left | KeyCode::Char('h') => app.cursor = (app.cursor + app.potions.len() - 1).rem_euclid(app.potions.len()),
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                app.on_tick();
                last_tick = Instant::now();
            }
        }
        restore_terminal()
    }

    fn on_tick(&mut self) {
        self.tick_count += 1;
        // only change marker every 180 ticks (3s) to avoid stroboscopic effect
        // if (self.tick_count % 180) == 0 {
        //     self.marker = match self.marker {
        //         Marker::Dot => Marker::Braille,
        //         Marker::Braille => Marker::Block,
        //         Marker::Block => Marker::HalfBlock,
        //         Marker::HalfBlock => Marker::Bar,
        //         Marker::Bar => Marker::Dot,
        //     };
        // }
        // bounce the ball by flipping the velocity vector
        let ball = &self.ball;
        let playground = self.playground;
        if ball.x - ball.radius < playground.left() as f64
            || ball.x + ball.radius > playground.right() as f64
        {
            self.vx = -self.vx;
        }
        if ball.y - ball.radius < playground.top() as f64
            || ball.y + ball.radius > playground.bottom() as f64
        {
            self.vy = -self.vy;
        }

        self.ball.x += self.vx;
        self.ball.y += self.vy;
    }

    fn ui(&self, frame: &mut Frame) {
        let percent = 100 / self.potions.len();
        // let constraint = Constraint::Percentage(percent as u16);
        let constraint = Constraint::Length(20);
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                std::iter::repeat(constraint)
                         .take(self.potions.len()))
            .margin(5)
            .flex(Flex::Center)
            .spacing(10)
            ;

        for (i, rect) in layout.split(frame.size()).into_iter().enumerate() {
            let selected = self.selected.map(|x| x == i).unwrap_or(false);

        let [gap, potion, footer] = Layout::vertical([
            Constraint::Min(if selected { 0 } else { 1 }),
            Constraint::Percentage(100),
            Constraint::Min(if selected { 2 } else { 1 })])
                .areas(*rect);
            frame.render_widget(self.potions[i].clone(), potion);

            frame.render_widget(Paragraph::new(if self.cursor == i { "/\\" } else { "" })
                                .alignment(Alignment::Center)
                                , footer);
        }

        // let horizontal =
        //     Layout::horizontal([Constraint::Percentage(50),
        //                         Constraint::Percentage(50)]);
        // let vertical = Layout::vertical([Constraint::Percentage(50),
        //                                  Constraint::Percentage(50)]);
        // let [map, right] = horizontal.areas(frame.size());
        // let [pong, boxes] = vertical.areas(right);

        // frame.render_widget(self.map_canvas(), map);
        // frame.render_widget(self.pong_canvas(), pong);
        // frame.render_widget(self.potion.clone(), boxes);
    }

    fn map_canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("World"))
            .marker(self.marker)
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::Green,
                    resolution: MapResolution::High,
                });
                ctx.print(self.x, -self.y, "You are here".yellow());
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
    }

    fn pong_canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("Pong"))
            .marker(self.marker)
            .paint(|ctx| {
                ctx.draw(&self.ball);
            })
            .x_bounds([10.0, 210.0])
            .y_bounds([10.0, 110.0])
    }

    // fn potion_canvas(&self, area: Rect) -> impl Widget + '_ {
    //     let left = 0.0;
    //     let right = f64::from(area.width);
    //     let bottom = 0.0;
    //     let top = f64::from(area.height);//.mul_add(2.0, -4.0);
    //     let center = right / 2.0;
    //     Canvas::default()
    //         .block(Block::bordered().title("Potion"))
    //         .marker(self.marker)
    //         .x_bounds([0., 100.])
    //         .y_bounds([bottom, 100.])
    //         .paint(|ctx| {

    //             ctx.draw(&Rectangle {
    //                 x: 50. - self.potion.width / 2.0,
    //                 y: 2.0,
    //                 width: self.potion.width,
    //                 height: self.potion.height,
    //                 color: Color::White,
    //             });
    //         })
    // }

    fn boxes_canvas(&self, area: Rect) -> impl Widget {
        let (left, right, bottom, top) =
            (0.0, area.width as f64, 0.0, area.height as f64 * 2.0 - 4.0);
        Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("Rects"))
            .marker(self.marker)
            .x_bounds([left, right])
            .y_bounds([bottom, top])
            .paint(|ctx| {
                for i in 0..=11 {
                    ctx.draw(&Rectangle {
                        x: (i * i + 3 * i) as f64 / 2.0 + 2.0,
                        y: 2.0,
                        width: i as f64,
                        height: i as f64,
                        color: Color::Red,
                    });
                    ctx.draw(&Rectangle {
                        x: (i * i + 3 * i) as f64 / 2.0 + 2.0,
                        y: 21.0,
                        width: i as f64,
                        height: i as f64,
                        color: Color::Blue,
                    });
                }
                for i in 0..100 {
                    if i % 10 != 0 {
                        ctx.print(i as f64 + 1.0, 0.0, format!("{i}", i = i % 10));
                    }
                    if i % 2 == 0 && i % 10 != 0 {
                        ctx.print(0.0, i as f64, format!("{i}", i = i % 10));
                    }
                }
            })
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
