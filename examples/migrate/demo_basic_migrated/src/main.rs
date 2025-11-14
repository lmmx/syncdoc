# ! [doc = syncdoc :: module_doc ! (path = "docs")]

use cli_demo_basic_migrated::TimeOfDay;

#[syncdoc::omnidoc(path = "docs")]
fn main() {
    let current = TimeOfDay::Day;

    match current {
        TimeOfDay::Day => println!("Good morning! It’s a bright new day."),
        TimeOfDay::Night => println!("Good evening! Time to relax under the stars."),
    }
}
