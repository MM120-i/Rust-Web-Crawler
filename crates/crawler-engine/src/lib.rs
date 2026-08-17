// Placeholder for the crawl loop (fetch/parse/enqueue orchestration, politeness scheduling).
// Real content lands in Phase 1; this keeps the crate compiling and testable in the meantime.
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
