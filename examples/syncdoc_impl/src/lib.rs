use syncdoc::omnidoc;

pub struct Calculator;

#[omnidoc(path = "../docs")]
impl Calculator {
    pub fn fibonacci(n: u64) -> u64 {
        if n <= 1 {
            n
        } else {
            Self::fibonacci(n - 1) + Self::fibonacci(n - 2)
        }
    }

    pub fn multiply(&self, a: u32, b: u32) -> u32 {
        a * b
    }

    pub fn add_numbers(&self, x: i32, y: i32) -> i32 {
        x + y
    }
}
