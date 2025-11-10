#![doc = include_str!("../docs/main.md")]

use syncdoc_struct::Settings;

fn main() {
    let opts = Settings { name: "FooBar 3000".to_string(), switch: true };

    println!("{}", format!("Running with {:?}", opts));
}
