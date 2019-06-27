use std::error::Error;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use termion::event::{Event as TEvent, Key};
use termion::input::TermRead;
use termion::AsyncReader;

use crate::app::{Event, State, ThreadControlMsg};
use crate::ui::InputBoxType;

pub fn input_loop(control: Receiver<ThreadControlMsg>, state: Arc<Mutex<State>>, stdin: AsyncReader, send: Sender<Event>) {
    let mut keys_iter = stdin.keys();
    loop {
        match control.try_recv() {
            Ok(msg) => match msg {
                ThreadControlMsg::Exit => break
            },
            _ => ()
        }
        while let Some(c) = keys_iter.next() {
            let mut state = state.lock().unwrap();
            let evt = c.unwrap();
            let result = match evt {
                // If Ctrl+c is input, we quit the application.
                Key::Ctrl('c') => {
                    send.send(Event::Exit).unwrap();
                    Ok(None)
                },
                // Left and right change the currently focused box.
                Key::Left => {
                    let selected_box = state.selected_box;
                    state.selected_box = match selected_box {
                        InputBoxType::EndX => InputBoxType::StartX,
                        _ => InputBoxType::Function,
                    };
                    Ok(None)
                },
                Key::Right => {
                    let selected_box = state.selected_box;
                    state.selected_box = match selected_box {
                        InputBoxType::Function => InputBoxType::StartX,
                        _ => InputBoxType::EndX,
                    };
                    Ok(None)
                },
                k => {
                    let selected_box = state.selected_box;
                    selected_box.handle_key(k, &mut state)
                }
            };
            if let Ok(Some(event)) = result {
                send.send(event).unwrap();
            }
        }
        thread::sleep(Duration::from_millis(1));
    }
}

type Handled = Result<Option<Event>, Box<Error>>;

trait InputHandler {
    fn handle_key(&self, key: Key, state: &mut State) -> Handled;
}

impl InputHandler for InputBoxType {
    fn handle_key(&self, key: Key, state: &mut State) -> Handled {
        match self {
            InputBoxType::StartX => handle_start_x_input(key, state),
            InputBoxType::Function => handle_fn_input(key, state),
            InputBoxType::EndX => handle_end_x_input(key, state),
        }
    }
}

fn handle_fn_input(key: Key, state: &mut State) -> Handled {
    match key {
        Key::Backspace => {
            state.function_input.pop();
            Ok(Some(Event::Update))
        }
        Key::Char(c) => {
            state.function_input.push(c);
            Ok(Some(Event::Update))
        }
        _ => Ok(None)
    }
}

fn handle_start_x_input(key: Key, state: &mut State) -> Handled {
    match key {
        Key::Up => {
            state.start_x_input = format!("{:+}", state.start_x + 1.0).to_string();
            state.start_x += 1.0;
            Ok(Some(Event::Update))
        }
        Key::Down => {
            state.start_x_input = format!("{:+}", state.start_x - 1.0).to_string();
            state.start_x -= 1.0;
            Ok(Some(Event::Update))
        }
        Key::Backspace => {
            // Reset to placeholder if our string is too short.
            if state.start_x_input.len() <= 2 {
                state.start_x_input = String::from("+0");
            } else {
                state.start_x_input.pop();
            }
            state.start_x = state.start_x_input.parse().unwrap();
            Ok(Some(Event::Update))
        }
        Key::Char(digit) if digit.is_ascii_digit() => {
            if &state.start_x_input == "+0" || &state.start_x_input == "-0" {
                state.start_x_input.pop();
            }
            state.start_x_input.push(digit);
            state.start_x = state.start_x_input.parse().unwrap();
            Ok(Some(Event::Update))
        }
        Key::Char('+') => {
            state.start_x_input.replace_range(..1, "+");
            state.start_x = state.start_x_input.parse().unwrap();
            Ok(Some(Event::Update))
        }
        Key::Char('-') => {
            state.start_x_input.replace_range(..1, "-");
            state.start_x = state.start_x_input.parse().unwrap();
            Ok(Some(Event::Update))
        }
        Key::Char('.') => {
            if !state.start_x_input.contains(".") {
                state.start_x_input.push('.');
            }
            state.start_x = state.start_x_input.parse().unwrap();
            Ok(Some(Event::Update))
        }
        _ => Ok(None),
    }
}

fn handle_end_x_input(key: Key, state: &mut State) -> Handled {
    match key {
        Key::Up => {
            state.end_x_input = format!("{:+}", state.end_x + 1.0).to_string();
            state.end_x += 1.0;
            Ok(Some(Event::Update))
        }
        Key::Down => {
            state.end_x_input = format!("{:+}", state.end_x - 1.0).to_string();
            state.end_x -= 1.0;
            Ok(Some(Event::Update))
        }
        Key::Backspace => {
            // Reset to placeholder if our string is too short.
            if state.end_x_input.len() <= 2 {
                state.end_x_input = String::from("+0");
            } else {
                state.end_x_input.pop();
            }
            state.end_x = state.end_x_input.parse().unwrap();
            Ok(Some(Event::Update))
        }
        Key::Char(digit) if digit.is_ascii_digit() => {
            if &state.end_x_input == "+0" || &state.end_x_input == "-0" {
                state.end_x_input.pop();
            }
            state.end_x_input.push(digit);
            state.end_x = state.end_x_input.parse().unwrap();
            Ok(Some(Event::Update))
        }
        Key::Char('+') => {
            state.end_x_input.replace_range(..1, "+");
            state.end_x = state.end_x_input.parse().unwrap();
            Ok(Some(Event::Update))
        }
        Key::Char('-') => {
            state.end_x_input.replace_range(..1, "-");
            state.end_x = state.end_x_input.parse().unwrap();
            Ok(Some(Event::Update))
        }
        Key::Char('.') => {
            if !state.end_x_input.contains(".") {
                state.end_x_input.push('.');
            }
            state.end_x = state.end_x_input.parse().unwrap();
            Ok(Some(Event::Update))
        }
        _ => Ok(None),
    }
}
