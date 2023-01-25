// Basic struct for a test
#[derive (Debug)]
pub struct Test {
    pub test_type: TestType,
    pub test_name: String,
    pub test_code: String
}

// Basic test types
#[derive (Debug, strum::Display)]
#[strum (serialize_all = "UPPERCASE")]
pub enum TestType {
    Lex
}