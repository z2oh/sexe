use io;

use nom::types::CompleteStr;

use termion::event;
use termion::input::TermRead;

use tui::backend::MouseBackend;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style};
use tui::widgets::*;
use tui::Terminal;

use std::collections::HashMap;

use parser;

#[derive(PartialEq, Eq)]
enum SelectedBox {
    Function,
    StartX,
    EndX,
}

struct Application {
    selected_box: SelectedBox,
    focused: bool,
    start_y: f64,
    end_y: f64,
    evaluation: Vec<(f64, f64)>,
    resolution: u32,
    function_input: TextInput,
    start_x_input: NumberInput,
    end_x_input: NumberInput,
}

struct TextInput {
    string: String,
    position: usize,
}

struct NumberInput {
    display_string: String,
    number_value: f64,
}

trait Input {
    fn process_input(&mut self, key: &event::Key);
}

impl Input for NumberInput {
    fn process_input(&mut self, key: &event::Key) {
        match key {
            event::Key::Backspace => {
                // Reset to placeholder if our string is too short.
                if self.display_string.len() <= 2 {
                    self.display_string = String::from("+0");
                }
                else {
                    self.display_string.pop();
                }
            },
            event::Key::Char(digit) if digit.is_ascii_digit() => {
                if &self.display_string == "+0" || &self.display_string == "-0" {
                    self.display_string.pop();
                }
                self.display_string.push(*digit);
            },
            event::Key::Char('+') => {
                self.display_string.replace_range(..1, "+");
            },
            event::Key::Char('-') => {
                self.display_string.replace_range(..1, "-");
            },
            event::Key::Char('.') => {
                if !self.display_string.contains(".") {
                    self.display_string.push('.');
                }
            },
            _ => (),
        };
        self.number_value = self.display_string.parse().unwrap();
    }
}

impl Input for TextInput {
    fn process_input(&mut self, key: &event::Key) {
        match key {
            event::Key::Backspace => {
                if self.position > 0 {
                    self.string.remove(self.position-1);
                    self.position-=1;
                }
            },
            event::Key::Char(c) => {
                self.string.insert(self.position, *c);
                self.position+=1;
            },
            event::Key::Left => {
                if self.position > 0 {
                    self.position-=1;
                }
            },
            event::Key::Right => {
                if self.position < self.string.len() {
                    self.position+=1;
                }
            },
            _ => {},
        };
    }
}

enum ApplicationOperation {
    Exit,
    Noop,
}

fn determine_y_bounds(vec: &Vec<(f64, f64)>) -> Option<(f64, f64)> {
    vec.into_iter().fold(None, |acc, &(_, y)| {
        Some(acc.map_or((y, y), |(acc_min, acc_max)| {
            (y.min(acc_min), y.max(acc_max))
        }))
    })
}

enum Error {
    ParseError,
    RangeError,
}

fn evaluate_function_over_domain(start_x: f64, end_x: f64, resolution: u32, function_string: &str) -> Result<Vec<(f64, f64)>, Error> {
    let mut vars_map = HashMap::new();
    vars_map.insert("x".to_string(), start_x);

    let func = match parser::parse_expr(CompleteStr(function_string)) {
        Ok((rem, func)) => {
            if rem.len() > 0 {
                return Err(Error::ParseError);
            }
            func
        },
        Err(_) => {
            return Err(Error::ParseError);
        },
    };

    let step_width = (end_x - start_x) / resolution as f64;

    Ok((0..resolution).map(|x| start_x + (x as f64 * step_width)).filter_map(|x| {
        if let Some(val) = vars_map.get_mut(&"x".to_string()) {
            *val = x;
        }
        match func.evaluate(&vars_map) {
            Ok(y) => Some((x, y)),
            Err(_) => None,
        }
    }).collect())
}

impl Application {
    fn process_input(&mut self, key: &event::Key) -> ApplicationOperation {
        match key {
            // A Ctrl-C produces an exit command for the application.
            event::Key::Ctrl('c') => return ApplicationOperation::Exit,
            // Left and right change the focused box.
            event::Key::Left if !self.focused => {
                self.selected_box = match self.selected_box {
                    SelectedBox::EndX => SelectedBox::StartX,
                    _ => SelectedBox::Function,
                };
            },
            event::Key::Right if !self.focused => {
                self.selected_box = match self.selected_box {
                    SelectedBox::Function => SelectedBox::StartX,
                    _ => SelectedBox::EndX,
                };
            },
            event::Key::Esc => {
                self.focused = false;
            },
            // Otherwise we potentially modify focus and hand off input to the children.
            _ => {
                match self.selected_box {
                SelectedBox::Function => {
                    self.focused = true;
                    self.function_input.process_input(&key)
                },
                SelectedBox::StartX => self.start_x_input.process_input(&key),
                SelectedBox::EndX => self.end_x_input.process_input(&key),
            }},
        };
        ApplicationOperation::Noop
    }

    fn draw(&self, t: &mut Terminal<MouseBackend>, size: &Rect) {
        Group::default()
            .direction(Direction::Vertical)
            .margin(1)
            .sizes(&[Size::Min(3), Size::Percent(100)])
            .render(t, size, |t, chunks| {
                Group::default()
                    .direction(Direction::Horizontal)
                    .sizes(&[Size::Percent(60), Size::Percent(20), Size::Percent(20)])
                    .render(t, &chunks[0], |t, chunks| {
                        Paragraph::default()
                            .block(
                                Block::default()
                                .title("Function")
                                .borders(Borders::ALL)
                                .border_style(self.get_box_style(SelectedBox::Function))
                            )
                            .style(self.get_input_style(SelectedBox::Function))
                            .wrap(false)
                            .text(&self.render_cursor(&self.function_input, self.focused))
                            .render(t, &chunks[0]);
                        Paragraph::default()
                            .block(
                                Block::default()
                                .title("Start X")
                                .borders(Borders::ALL)
                                .border_style(self.get_box_style(SelectedBox::StartX))
                            )
                            .style(self.get_input_style(SelectedBox::StartX))
                            .wrap(false)
                            .text(&self.start_x_input.display_string)
                            .render(t, &chunks[1]);
                        Paragraph::default()
                            .block(
                                Block::default()
                                .title("End X")
                                .borders(Borders::ALL)
                                .border_style(self.get_box_style(SelectedBox::EndX))
                            )
                            .style(self.get_input_style(SelectedBox::EndX))
                            .wrap(false)
                            .text(&self.end_x_input.display_string)
                            .render(t, &chunks[2]);
                    });
                Chart::default()
                    .block(Block::default().title("Plot").borders(Borders::ALL))
                    .x_axis(Axis::default()
                        .title("X")
                        .bounds([self.start_x_input.number_value, self.end_x_input.number_value])
                        .labels(&[
                                format!("{:.2}", self.start_x_input.number_value).as_str(),
                                "0",
                                format!("{:.2}", self.end_x_input.number_value).as_str(),
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
        self.resolution = (term_size.width * 3).into();

        match self.plot_function() {
            Ok(vec) => {
                // Filters all instances of f64::NAN from the vector
                self.evaluation = vec.into_iter()
                    .filter(|&(_,a)| a.is_normal()).collect();
                let (start_y, end_y) =
                    determine_y_bounds(&self.evaluation).unwrap_or((0.0, 0.0));
                if start_y == end_y {
                    let end_y_abs = end_y.abs();
                    self.start_y = -end_y_abs;
                    self.end_y = end_y_abs;
                } else {
                    self.start_y = start_y;
                    self.end_y = end_y;
                }
            },
            Err(_) => {
                self.evaluation = Vec::new();
                self.start_y = 0.0;
                self.end_y = 0.0;
            },
        }

        self.draw(&mut terminal, &term_size);

        for c in stdin.keys() {
            let size = terminal.size().unwrap();
            if term_size != size {
                terminal.resize(size).unwrap();
                term_size = size;
                self.resolution = (term_size.width * 3).into();
            }
            let evt = c.unwrap();

            match self.process_input(&evt) {
                ApplicationOperation::Exit => break,
                ApplicationOperation::Noop => (),
            };

            // TODO: Handle plotting errors and display error messages.
            match self.plot_function() {
                Ok(vec) => {
                    // Filters all instances of f64::NAN from the vector
                    self.evaluation = vec.into_iter()
                        .filter(|&(_,a)| a.is_normal()).collect();
                    let (start_y, end_y) =
                        determine_y_bounds(&self.evaluation).unwrap_or((0.0, 0.0));
                    if start_y == end_y {
                        let end_y_abs = end_y.abs();
                        self.start_y = -end_y_abs;
                        self.end_y = end_y_abs;
                    } else {
                        self.start_y = start_y;
                        self.end_y = end_y;
                    }
                },
                Err(_) => {
                    self.evaluation = Vec::new();
                    self.start_y = 0.0;
                    self.end_y = 0.0;
                },
            }

            self.draw(&mut terminal, &term_size);
        }
        terminal.clear().unwrap();
        terminal.show_cursor().unwrap();
    }

    fn plot_function(&mut self) -> Result<Vec<(f64, f64)>, Error> {
        if self.start_x_input.number_value >= self.end_x_input.number_value {
            Err(Error::RangeError)
        }
        else {
            evaluate_function_over_domain(self.start_x_input.number_value, self.end_x_input.number_value, self.resolution, &self.function_input.string)
        }
    }

    fn get_input_style(&self, _selected: SelectedBox) -> Style {
        // leaving this method as a reference how to change the text of focused input
        Style::default()
    }

    fn get_box_style(&self, selected: SelectedBox) -> Style {
        if selected == self.selected_box {
            Style::default().fg(Color::Magenta) 
        } else {
            Style::default().fg(Color::Gray)
        }
    }

    fn render_cursor(&self, input: &TextInput, _focused: bool) -> String {
        // Renders a rudimentary cursor
        let mut cursor_input = input.string.clone();
        if _focused {
            cursor_input.insert(input.position, '|');
        }
        return cursor_input;
    }
}

pub fn display() {
    let mut application = Application {
        selected_box: SelectedBox::Function,
        focused: false,
        start_y: 0.0,
        end_y: 0.0,
        evaluation: Vec::new(),
        function_input: TextInput {
            string: String::from("sin(x)"),
            position: 6, // Hard-coded as the length of the default string
        },
        start_x_input: NumberInput {
            display_string: String::from("+0"),
            number_value: 0.0,
        },
        end_x_input: NumberInput {
            display_string: String::from("+10"),
            number_value: 10.0,
        },
        resolution: 100,
    };
    application.start();
}
#[cfg(test)]
mod tests {
    use super::*;
    extern crate test;
    use self::test::Bencher;

    #[bench]
    fn single_variable_100res(b: &mut Bencher) {
        b.iter(|| {
            evaluate_function_over_domain(0.0f64, 10.0f64, 100, "x")
        });
    }

    #[bench]
    fn sine_expression_100res(b: &mut Bencher) {
        b.iter(|| {
            evaluate_function_over_domain(0.0f64, 10.0f64, 100, "sin(x)")
        });
    }

    #[bench]
    fn complicated_expression_100res(b: &mut Bencher) {
        b.iter(|| {
            evaluate_function_over_domain(0.0f64, 10.0f64, 100, "|sin(x^x) / 2^((x^x - pi/2)/pi)|")
        });
    }

    #[bench]
    fn single_variable_amillires(b: &mut Bencher) {
        b.iter(|| {
            evaluate_function_over_domain(0.0f64, 10.0f64, 1000000, "x")
        });
    }

    #[bench]
    fn sine_expression_amillires(b: &mut Bencher) {
        b.iter(|| {
            evaluate_function_over_domain(0.0f64, 10.0f64, 1000000, "sin(x)")
        });
    }

    #[bench]
    fn complicated_expression_amillires(b: &mut Bencher) {
        b.iter(|| {
            evaluate_function_over_domain(0.0f64, 10.0f64, 1000000, "|sin(x^x) / 2^((x^x - pi/2)/pi)|")
        });
    }


}