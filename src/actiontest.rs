// solely exists for now so that the github action for tests can run
pub fn hello_world() -> String {
    String::from("Hello, world!")
}

// unit testing will happen in respective files as to have access to private functions
#[cfg(test)] 
mod tests {
    use super::*;

    #[test]
    fn test_hello_world(){
        assert_eq!(hello_world(), "Hello, world!")
    }
}