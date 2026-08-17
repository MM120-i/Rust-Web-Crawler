// Placeholder for shared domain logic (URL scope rules, politeness policy, frontier types).
// Real content lands in Phase 1; this keeps the crate compiling and testable in the meantime.
pub fn hello_world() -> String {
    String::from("Hello, world!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello, world!")
    }
}
