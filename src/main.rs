#[macro_use]
extern crate nom;
extern crate termion;
extern crate tui;

use std::io;

mod expression;
mod parser;
mod interface;

fn main() {
    let should_display_interface = true;
    if should_display_interface {
        // Display the interface and hand control over to `display` module.
        interface::display();
    }
    else {
        // Do nothing, eventually other options will be added.
        return;
    }
}

