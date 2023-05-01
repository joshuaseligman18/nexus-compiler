use crate::util::{nexus_log, target::Target};
use crate::nexus::{lexer::Lexer, token::Token, parser::Parser, semantic_analyzer::SemanticAnalyzer, syntax_tree::SyntaxTree};
use crate::nexus::code_generator_6502::CodeGenerator6502;
use crate::nexus::code_generator_riscv::CodeGeneratorRiscV;
use crate::editor::buttons;

// Function to compile multiple programs
pub fn compile(source_code: &str) {
    let mut lexer: Lexer = Lexer::new(source_code);
    let mut parser: Parser = Parser::new();
    let mut semantic_analyzer: SemanticAnalyzer = SemanticAnalyzer::new();
    let mut code_generator_6502: CodeGenerator6502 = CodeGenerator6502::new();
    let mut code_generator_riscv: CodeGeneratorRiscV = CodeGeneratorRiscV::new();

    // Clean up the output area
    SyntaxTree::clear_display();
    CodeGenerator6502::clear_display();
    nexus_log::clear_logs();
    nexus_log::log(
        nexus_log::LogTypes::Info,
        nexus_log::LogSources::Nexus,
        String::from("Nexus compile called")
    );

    // Keep track of the number of programs
    let mut program_number: u32 = 0;

    // Go through each program
    while lexer.has_program_to_lex() {
        program_number += 1;

        nexus_log::insert_empty_line();

        // Log the program we are on
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Nexus,
            format!("Compiling program {}", program_number)
        );
        nexus_log::insert_empty_line();

        // Log the program we are lexing
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Lexer,
            format!("Lexing program {}", program_number)
        );

        // Lex the program
        let lex_res: Result<Vec<Token>, ()> = lexer.lex_program();

        nexus_log::insert_empty_line();

        if lex_res.is_err() {
            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Parser,
                String::from("Parsing skipped due to lex failure")
            );

            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("CST display skipped due to lex failure")
            );
            
            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("AST generation and display skipped due to lex failure")
            );

            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::SemanticAnalyzer,
                String::from("Semantic analysis skipped due to lex failure")
            );

            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("Symbol table display skipped due to lex failure")
            );
            
            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("Code generation skipped due to lex failure")
            );

            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("Executable image display skipped due to lex failure")
            );

            // No need to move on if lex failed, so can go to next program
            continue;
        }

        // Log the program we are lexing
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Parser,
            format!("Parsing program {}", program_number)
        );

        let token_stream: Vec<Token> = lex_res.unwrap();
        let parse_res: Result<SyntaxTree, ()> = parser.parse_program(&token_stream);

        if parse_res.is_err() {
            nexus_log::insert_empty_line();

            // Do not show CST unless parse is successful
            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("CST display skipped due to parse failure")
            );
            
            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("AST generation and display skipped due to parse failure")
            );

            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::SemanticAnalyzer,
                String::from("Semantic analysis skipped due to parse failure")
            );

            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("Symbol table display skipped due to parse failure")
            );

            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("Code generation skipped due to parse failure")
            );

            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("Executable image display skipped due to parse failure")
            );

            continue;
        }

        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Nexus,
            format!("CST display for program {} is below", program_number)
        );
        let cst: SyntaxTree = parse_res.unwrap();
        cst.display(&program_number);

        nexus_log::insert_empty_line();
        
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Nexus,
            format!("Generating AST for program {}", program_number)
        );

        let ast: SyntaxTree = semantic_analyzer.generate_ast(&token_stream);
        ast.display(&program_number);

        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Nexus,
            format!("AST display for program {} is below", program_number)
        );

        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::SemanticAnalyzer,
            format!("Beginning semantic analysis on program {}", program_number)
        );
        let semantic_analysis_res: bool = semantic_analyzer.analyze_program(&ast);

        if !semantic_analysis_res {
            nexus_log::insert_empty_line();

            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("Symbol table display skipped due to semantic analysis failure")
            );
            
            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("Code generation skipped due to semantic analysis failure")
            );

            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("Executable image display skipped due to semantic analysis failure")
            );

            continue;
        }

        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Nexus,
            format!("Symbol table for program {} is below", program_number)
        );
        semantic_analyzer.symbol_table.display_symbol_table(&program_number);

        nexus_log::insert_empty_line();

        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::CodeGenerator,
            format!("Generating code for program {}", program_number)
        );
       
        match buttons::get_current_target() {
            Target::Target6502 => code_generator_6502.generate_code(&ast, &mut semantic_analyzer.symbol_table, &program_number),
            Target::TargetRiscV => code_generator_riscv.generate_code(&ast, &mut semantic_analyzer.symbol_table, &program_number)
        }
    }
}
