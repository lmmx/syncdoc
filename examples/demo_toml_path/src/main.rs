#![doc = include_str!("../docs/main.md")]

use demo_toml_path::Settings;

fn main() {
    let opts = Settings { name: "FooBar 3000".to_string(), switch: true };

    println!("Running with {:?}", opts);
}
