use log::info;

use crate::util::nexus_log;
use crate::nexus::{lexer, token::Token};

pub fn compile(source_code: String) {
    nexus_log::clear_logs();
    nexus_log::log(
        nexus_log::LogTypes::Info,
        nexus_log::Sources::Nexus,
        String::from("Compile called")
    );
    let mut token_stream: Vec<Token> = lexer::lex(&source_code);
    info!("{:?}: {}", token_stream, token_stream.len());
}