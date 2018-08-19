use io;

use nom::types::CompleteStr;

use termion::event;
use termion::input::TermRead;
use termion::cursor;

use tui::backend::MouseBackend;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style};
use tui::widgets::*;
use tui::Terminal;

use std::collections::HashMap;

use parser;

enum InputMode {
    Normal,
    Insert,
}

enum SelectedBox {
    Function,
    StartX,
    EndX,
}

struct Application {
    mode: InputMode,
    function_string: String,
    selected_box: SelectedBox,
    start_x_string: String,
    end_x_string: String,
    start_x: f64,
    end_x: f64,
    start_y: f64,
    end_y: f64,
    evaluation: Vec<(f64, f64)>,
}

impl Application {
    fn draw(&self, t: &mut Terminal<MouseBackend>, size: &Rect) {
        Group::default()
            .direction(Direction::Vertical)
            .margin(1)
            .sizes(&[Size::Percent(10), Size::Percent(90)])
            .render(t, size, |t, chunks| {
                Group::default()
                    .direction(Direction::Horizontal)
                    .sizes(&[Size::Percent(60), Size::Percent(20), Size::Percent(20)])
                    .render(t, &chunks[0], |t, chunks| {
                        Paragraph::default()
                            .block(Block::default().title("Function").borders(Borders::ALL))
                            .style(Style::default())
                            .wrap(false)
                            .text(&self.function_string)
                            .render(t, &chunks[0]);
                        Paragraph::default()
                            .block(Block::default().title("Start X").borders(Borders::ALL))
                            .style(Style::default())
                            .wrap(false)
                            .text(&self.start_x_string)
                            .render(t, &chunks[1]);
                        Paragraph::default()
                            .block(Block::default().title("End X").borders(Borders::ALL))
                            .style(Style::default())
                            .wrap(false)
                            .text(&self.end_x_string)
                            .render(t, &chunks[2]);
                    });
                Chart::default()
                    .block(Block::default().title("Plot").borders(Borders::ALL))
                    .x_axis(Axis::default()
                        .title("X")
                        .bounds([self.start_x, self.end_x])
                        .labels(&[
                                format!("{:.2}", self.start_x).as_str(),
                                "0",
                                format!("{:.2}", self.end_x).as_str(),
                        ]))
                    .y_axis(Axis::default()
                        .title("Y")
                        .bounds([self.start_y, self.end_y])
                        .labels(&[
                                format!("{:.2}", self.start_y).as_str(),
                                "0",
                                format!("{:.2}", self.end_y).as_str(),
                        ]))
                    .datasets(&[Dataset::default()
                                .name("data")
                                .marker(Marker::Braille)
                                .style(Style::default().fg(Color::Magenta))
                                .data(&self.evaluation)])
                    .render(t, &chunks[1]);
            });

        t.draw().unwrap();
    }

    fn start(&mut self) {
        let stdin = io::stdin();
        let mut terminal = Terminal::new(MouseBackend::new().unwrap()).unwrap();
        terminal.clear().unwrap();
        terminal.hide_cursor().unwrap();

        let mut term_size = terminal.size().unwrap();
        self.draw(&mut terminal, &term_size);

        for c in stdin.keys() {
            let size = terminal.size().unwrap();
            if term_size != size {
                terminal.resize(size).unwrap();
                term_size = size;
            }
            let evt = c.unwrap();
            match evt {
                event::Key::Ctrl('c') => break,
                event::Key::Backspace => {
                    match self.selected_box {
                        SelectedBox::Function => {
                            self.function_string.pop();
                        },
                        SelectedBox::StartX => {
                            self.start_x_string.pop();
                            self.start_x = match self.start_x_string.parse() {
                                Ok(val) => val,
                                Err(_) => {
                                    self.start_x_string = String::from("0");
                                    0.0
                                },
                            };
                        },
                        SelectedBox::EndX => {
                            self.end_x_string.pop();
                            self.end_x = match self.end_x_string.parse() {
                                Ok(val) => val,
                                Err(_) => {
                                    self.end_x_string = String::from("0");
                                    0.0
                                },
                            };
                        },
                    };
                },
                event::Key::Left => {
                    self.selected_box = match self.selected_box {
                        SelectedBox::EndX => SelectedBox::StartX,
                        _ => SelectedBox::Function,
                    };
                },
                event::Key::Right => {
                    self.selected_box = match self.selected_box {
                        SelectedBox::Function => SelectedBox::StartX,
                        _ => SelectedBox::EndX,
                    };
                },
                event::Key::Char(a) => { 
                    match self.selected_box {
                        SelectedBox::Function => {
                            self.function_string.push(a);
                        },
                        SelectedBox::StartX => {
                            if self.start_x_string.len() == 1 && self.start_x_string.starts_with("0") {
                                self.start_x_string.pop();
                            }
                            self.start_x_string.push(a);
                            self.start_x = match self.start_x_string.parse() {
                                Ok(val) => val,
                                Err(_) => {
                                    self.start_x_string = String::from("0");
                                    0.0
                                },
                            };
                        },
                        SelectedBox::EndX => {
                            if self.end_x_string.len() == 1 && self.end_x_string.starts_with("0") {
                                self.end_x_string.pop();
                            }
                            self.end_x_string.push(a);
                            self.end_x = match self.end_x_string.parse() {
                                Ok(val) => val,
                                Err(_) => {
                                    self.end_x_string = String::from("0");
                                    0.0
                                },
                            };
                        },
                    };
                },
                _ => continue,
            };
            self.evaluation = self.determine_evaluation(100);
            let (start_y, end_y) = self.determine_y_bounds();
            self.start_y = start_y;
            self.end_y = end_y;
            self.draw(&mut terminal, &term_size);
        }
        terminal.clear().unwrap();
        terminal.show_cursor().unwrap();
    }

    fn determine_y_bounds(&self) -> (f64, f64) {
        let mut current_min = 0.0;
        let mut current_max = 0.0;
        for (x, y) in &self.evaluation {
            current_min = if *y < current_min { *y } else { current_min };
            current_max = if *y > current_max { *y } else { current_max };
        }
        (current_min, current_max)
    }

    fn determine_evaluation(&self, resolution: u32) -> Vec<(f64, f64)> {
        if self.start_x >= self.end_x {
            return Vec::new();
        }
        let mut vars_map = HashMap::new();
        vars_map.insert("x".to_string(), self.start_x);
        let func = match parser::parse_expr(CompleteStr(&self.function_string)) {
            Ok((rem, func)) => {
                if rem.len() > 0 {
                    return Vec::new();
                }
                func
            },
            Err(_) => {
                return Vec::new();
            },
        };
        let step_width = (self.end_x - self.start_x) / resolution as f64;

        (0..resolution).map(|x| self.start_x + (x as f64 * step_width)).filter_map(|x| {
            if let Some(val) = vars_map.get_mut(&"x".to_string()) {
                *val = x;
            }
            match func.evaluate(&vars_map) {
                Ok(y) => Some((x, y)),
                Err(e) => None,
            }
        }).collect()
    }
}

pub fn display() {
    let mut application = Application {
        mode: InputMode::Normal,
        function_string: String::from("sin(x)"),
        selected_box: SelectedBox::Function,
        start_x_string: String::from("0"),
        end_x_string: String::from("0"),
        start_x: 0.0,
        end_x: 0.0,
        start_y: 0.0,
        end_y: 0.0,
        evaluation: Vec::new(),
    };
    application.start();
}

