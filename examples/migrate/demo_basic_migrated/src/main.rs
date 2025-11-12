//! A main.rs module with a main function that prints good morning

use cli_demo_basic_migrated::TimeOfDay;

/// A function that prints a hello world message.
fn main() {
    let current = TimeOfDay::Day;

    match current {
        TimeOfDay::Day => println!("Good morning! It’s a bright new day."),
        TimeOfDay::Night => println!("Good evening! Time to relax under the stars."),
    }
}
