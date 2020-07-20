#[allow(dead_code)]
mod bindings;
mod mg_value;
mod connection;
mod error;

pub use connection::*;
pub use mg_value::*;
pub use error::*;

pub fn add_two(a: i32) -> i32 {
    internal_adder(a, 2)
}

fn internal_adder(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        assert_eq!(4, internal_adder(2, 2));
    }
}
