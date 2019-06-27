use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::layout::*;
use tui::style::{Color, Style};
use tui::Terminal;
use tui::terminal::Frame;
use tui::widgets::*;

use crate::app::{State, ThreadControlMsg};

// We want to render at 60 fps, so we want to render every 16 ms.
const FRAME_TIME_MS: u64 = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputBoxType {
    Function,
    StartX,
    EndX,
}

impl State {
    /// Returns the desired state for the input box.
    fn get_box_style(&self, selected: InputBoxType) -> Style {
        if selected == self.selected_box {
            Style::default().fg(Color::Magenta)
        } else {
            Style::default().fg(Color::Gray)
        }
    }
}

pub fn render_loop<B: Backend>(control: Receiver<ThreadControlMsg>, state: Arc<Mutex<State>>, terminal: Arc<Mutex<Terminal<B>>>) {
    loop {
        match control.try_recv() {
            Ok(msg) => match msg {
                ThreadControlMsg::Exit => break
            },
            _ => ()
        }
        let render_start = Instant::now();
        {
            // Lock the app_state so we can access the data.
            let mut state = state.lock().unwrap();
            // Lock the terminal so that we may draw to it.
            let mut terminal = terminal.lock().unwrap();
            render_ui(&state, &mut terminal);
            let term_size = terminal.size().unwrap();
            // Update the resolution based on the terminal size.
            state.resolution = (term_size.width * 3).into();
            // The mutexes are unlocked here when we go out of scope.
        }
        let elapsed_time = Instant::now().duration_since(render_start);
        // Sleep until time to draw the next frame.
        if elapsed_time < Duration::from_millis(FRAME_TIME_MS) {
            thread::sleep(Duration::from_millis(FRAME_TIME_MS) - elapsed_time);
        }
    }

    let mut terminal = terminal.lock().unwrap();
    // Reshow the cursor.
    terminal.show_cursor().unwrap();
}

fn render_ui<B: Backend>(state: &State, t: &mut Terminal<B>) {
    t.draw(|mut f: Frame<B>| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(vec![Constraint::Min(3), Constraint::Percentage(100)])
            .split(f.size());

        let input_section = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(60),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .direction(Direction::Horizontal)
            .split(chunks[0]);

        Paragraph::new([Text::raw(&state.function_input)].iter())
            .block(
                Block::default()
                    .title("Function")
                    .borders(Borders::ALL)
                    .border_style(state.get_box_style(InputBoxType::Function)),
            )
            .style(Style::default())
            .wrap(false)
            .render(&mut f, input_section[0]);

        Paragraph::new([Text::raw(&state.start_x_input)].iter())
            .block(
                Block::default()
                    .title("Start X")
                    .borders(Borders::ALL)
                    .border_style(state.get_box_style(InputBoxType::StartX)),
            )
            .style(Style::default())
            .wrap(false)
            .render(&mut f, input_section[1]);

        Paragraph::new([Text::raw(&state.end_x_input)].iter())
            .block(
                Block::default()
                    .title("End X")
                    .borders(Borders::ALL)
                    .border_style(state.get_box_style(InputBoxType::EndX)),
            )
            .style(Style::default())
            .wrap(false)
            .render(&mut f, input_section[2]);

        Chart::default()
            .block(Block::default().title("Plot").borders(Borders::ALL))
            .x_axis(
                Axis::default()
                    .title("X")
                    .bounds([ state.start_x, state.end_x, ])
                    .labels(&[
                        format!("{:.2}", state.start_x).as_str(),
                        "0",
                        format!("{:.2}", state.end_x).as_str(),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .title("Y")
                    .bounds([state.start_y, state.end_y, ])
                    .labels(&[
                        format!("{:.2}", state.start_y).as_str(),
                        "0",
                        format!("{:.2}", state.end_y).as_str(),
                    ]),
            )
            .datasets(&[Dataset::default()
                .marker(Marker::Braille)
                .style(Style::default().fg(Color::Magenta))
                .data(&state.evaluation)])
            .render(&mut f, chunks[1]);
    }).unwrap();
}
