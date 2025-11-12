#![doc = include_str!("../docs/main.md")]

use demo_cfg_attr_call::TimeOfDay;

fn main() {
    let current = TimeOfDay::Day;

    match current {
        TimeOfDay::Day => println!("Good morning! It’s a bright new day."),
        TimeOfDay::Night => println!("Good evening! Time to relax under the stars."),
    }
}
