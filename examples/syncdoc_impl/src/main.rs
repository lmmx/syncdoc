use syncdoc_impl::*;

fn main() {
    println!("=== Testing syncdoc on impl blocks ===");

    let calc = Calculator;

    println!("\n1. fibonacci(5):");
    let result = Calculator::fibonacci(5);
    println!("fibonacci(5) = {}", result);

    println!("\n2. multiply(6, 7):");
    let product = calc.multiply(6, 7);
    println!("multiply(6, 7) = {}", product);

    println!("\n3. add_numbers(10, 20):");
    let sum = calc.add_numbers(10, 20);
    println!("add_numbers(10, 20) = {}", sum);
}
