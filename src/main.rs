extern crate sexe_parser;
extern crate sexe_expression;
extern crate termion;
extern crate tui;

use std::io;

mod app;
mod ui;
mod input;

fn main() -> Result<(), Box<std::error::Error>> {
    let should_display_interface = true;
    if should_display_interface {
        // Display the interface and hand control over to `display` module.
        app::start()
    } else {
        // Do nothing, eventually other options will be added.
        Ok(())
    }
}
