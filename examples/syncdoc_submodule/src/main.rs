#![doc = include_str!("../docs/main.md")]
use syncdoc_submodule::*;

fn main() {
    println!("=== Testing syncdoc ===");

    println!("\n1. fibonacci(5):");
    let result = fibonacci(5);
    println!("fibonacci(5) = {}", result);

    println!("\n2. multiply(6, 7):");
    let product = multiply(6, 7);
    println!("multiply(6, 7) = {}", product);

    println!("\n3. add_numbers(10, 20):");
    let sum = add_numbers(10, 20);
    println!("add_numbers(10, 20) = {}", sum);
}
