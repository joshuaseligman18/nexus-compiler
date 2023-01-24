use crate::util::nexus_log;
use crate::nexus::lexer;

pub fn compile(source_code: String) {
    nexus_log::clear_logs();
    nexus_log::info(String::from("COMPILER"), String::from("Compile called"));
    lexer::lex(source_code);
}