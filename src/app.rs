use io;

use std::error::Error;
use std::fmt;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use crate::ui::{InputBoxType};

use sexe_expression as expression;
use sexe_parser as parser;

pub struct State {
    pub selected_box: InputBoxType,
    pub function_input: String,
    pub start_x_input: String,
    pub end_x_input: String,
    pub start_x: f64,
    pub end_x: f64,
    pub start_y: f64,
    pub end_y: f64,
    pub resolution: u32,
    pub evaluation: Vec<(f64, f64)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selected_box: InputBoxType::Function,
            function_input: String::from("sin(x)"),
            start_x_input: String::from("+0"),
            end_x_input: String::from("+10"),
            start_x: 0.0,
            end_x: 10.0,
            start_y: 0.0,
            end_y: 0.0,
            resolution: 0,
            evaluation: Vec::new(),
        }
    }
}

pub enum ThreadControlMsg {
    Exit,
}

pub enum Event {
    Exit,
    Update,
}

#[derive(Debug)]
pub enum UpdateError {
    RangeError,
    ParseError,
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpdateError::RangeError => write!(f, "RangeError"),
            UpdateError::ParseError => write!(f, "RangeError"),
        }
    }
}

impl Error for UpdateError {
    fn description(&self) -> &str {
        match self {
            UpdateError::RangeError => "RangeError",
            UpdateError::ParseError => "ParseError",
        }
    }
}

pub fn start() -> Result<(), Box<Error>> {
    // Obtain a handle to raw stdout.
    let stdout = io::stdout().into_raw_mode()?;
    // Use termion to create an alternate screen that we will render to.
    let screen = AlternateScreen::from(stdout);
    // We specify the use of termion as our tui backend.
    let backend = TermionBackend::new(screen);
    //let backend = TermionBackend::new(stdout);
    // Construct the terminal object.
    let mut terminal = Terminal::new(backend)?;
    // Hiding the cursor for now; we can hopefully display some kind of custom
    // cursor in the UI eventually.
    terminal.hide_cursor().unwrap();

    let mut state = State::default();
    // Set the starting resolution; this will be updated by the ui thread after
    // initializaton.
    let term_size = terminal.size().unwrap();
    // Update the resolution based on the terminal size.
    state.resolution = (term_size.width * 3).into();

    let arc_term = Arc::new(Mutex::new(terminal));
    let arc_state = Arc::new(Mutex::new(state));

    let stdin = termion::async_stdin();

    let (in_send, in_rec) = channel();
    let in_state = arc_state.clone();
    let (in_control_send, in_control_rec) = channel();

    let in_handle = thread::Builder::new().name("input".into()).spawn(move || {
        crate::input::input_loop(in_control_rec, in_state, stdin, in_send);
    }).unwrap();

    let ui_state = arc_state.clone();
    let ui_term = arc_term.clone();
    let (ui_control_send, ui_control_rec) = channel();

    let ui_handle = thread::Builder::new().name("ui".into()).spawn(move || {
        crate::ui::render_loop(ui_control_rec, ui_state, ui_term);
    }).unwrap();

    {
        let mut state = arc_state.lock().unwrap();
        update(&mut state);
    }

    loop {
        let evt_msg = in_rec.recv()?;
        match evt_msg {
            Event::Update => {
                let mut state = arc_state.lock().unwrap();
                update(&mut state);
            },
            Event::Exit => break,
        }
        thread::sleep(Duration::from_millis(1));
    }

    // Send kill messages on control channels to threads.
    ui_control_send.send(ThreadControlMsg::Exit)?;
    in_control_send.send(ThreadControlMsg::Exit)?;

    // Wait for the threads to finish.
    in_handle.join().unwrap();
    ui_handle.join().unwrap();

    Ok(())
}

fn update(state: &mut State) -> Result<(), Box<Error>> {
    if state.start_x >= state.end_x {
        Err(Box::new(UpdateError::RangeError))
    } else {
        if let Ok(func) = parser::parse(&state.function_input) {
            let vec = expression::evaluate(state.start_x, state.end_x, state.resolution, &func)
                .into_iter()
                .filter(|(x, y)| !x.is_nan() && !y.is_nan())
                .collect::<Vec<(f64, f64)>>();

            let (y_min, y_max) = determine_y_bounds(&vec).unwrap_or((0.0, 0.0));
            state.evaluation = vec;
            state.start_y = y_min;
            state.end_y = y_max;
            Ok(())
        } else {
            Err(Box::new(UpdateError::ParseError))
        }
    }
}

fn determine_y_bounds(vec: &[(f64, f64)]) -> Option<(f64, f64)> {
    vec.into_iter().fold(None, |acc, &(_, y)| {
        Some(acc.map_or((y, y), |(acc_min, acc_max)| {
            (y.min(acc_min), y.max(acc_max))
        }))
    })
}
