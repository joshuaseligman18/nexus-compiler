use log::*;

use crate::nexus::{syntax_tree::SyntaxTree, syntax_tree_node::*, symbol_table::*};
use crate::nexus::token::{TokenType, Keywords};
use crate::util::nexus_log;
use petgraph::graph::{NodeIndex};

use std::collections::HashMap;
use std::fmt;
use web_sys::{Document, Window, Element, DomTokenList};
use wasm_bindgen::{prelude::Closure, JsCast};
use wasm_bindgen::prelude::*;

use string_builder::Builder;

// Have to import the editor js module
#[wasm_bindgen(module = "/editor.js")]
extern "C" {
    // Import the getCodeInput function from js so we can call it from the Rust code
    #[wasm_bindgen(js_name = "setClipboard")]
    fn set_clipboard(newText: &str);
}

// The struct for the code generator
#[derive (Debug)]
pub struct CodeGeneratorRiscV {
    // The current max scope we have seen so far, which are encountered in
    // sequential order
    max_scope: usize,
    
    // The array for the code
    code_arr: Vec<String>,

    // The array for the variables
    static_arr: Vec<String>,
    
    // The array for strings / heap data
    heap_arr: Vec<String>,

    // Index for the temoprary data
    temp_index: usize,

    // Hashmap to keep track of the strings being stored on the heap
    string_history: HashMap<String, usize>,

    // The number of if statements
    if_count: usize,

    // The number of while statements
    while_count: usize
}

impl CodeGeneratorRiscV {
    pub fn new() -> Self {
        return CodeGeneratorRiscV {
            max_scope: usize::MAX,
            code_arr: Vec::new(),
            static_arr: Vec::new(),
            heap_arr: Vec::new(),
            temp_index: 0,
            string_history: HashMap::new(),
            if_count: 0,
            while_count: 0
        };
    }

    pub fn generate_code(&mut self, ast: &SyntaxTree, symbol_table: &mut SymbolTable, program_number: &u32) {
        // Make sure the current scope is set to be a flag for none
        self.max_scope = usize::MAX;
        
        self.code_arr.clear();
        self.static_arr.clear();
        self.heap_arr.clear();

        // Initialize the basic data for printing functionality
        self.heap_arr.push(format!("new_line: .ascii \"\\n\""));
        self.heap_arr.push(format!("print_int_char: .byte 0"));
        
        self.temp_index = 0;
        self.string_history.clear();
        self.if_count = 0;
        self.while_count = 0;

        // Store the actual strings "true" and "false"
        self.store_string("false");
        self.store_string("true");

        // Generate the code for the program
        let program_res: bool = self.code_gen_block(ast, NodeIndex::new((*ast).root.unwrap()), symbol_table);
        
        // Add the code to exit the program
        self.code_arr.push(format!("li  a7, 93"));
        self.code_arr.push(format!("li  a0, 0"));
        self.code_arr.push(format!("ecall"));

        // Add a function for printing an integer
        self.add_print_int_code();
        self.add_print_string_code();
        self.add_print_boolean_code();
        self.add_print_new_line_code();
        self.add_compare_eq_code();
        self.add_compare_neq_code();
       
        debug!("{:?}", self.code_arr);
        debug!("{:?}", self.static_arr);
        debug!("{:?}", self.heap_arr);

        info!("{}", self.create_output_string());

        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::CodeGenerator,
            format!("Code generation completed successfully")
        );

        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Nexus,
            format!("Executable image for program {} is below", *program_number)
        );

        self.display_code(program_number);
    }

    fn code_gen_block(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) -> bool {
        // If this is the first block, then the first scope is 0
        if self.max_scope == usize::MAX {
            self.max_scope = 0;
        } else {
            // Otherwise just add 1
            self.max_scope += 1;
        }

        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::CodeGenerator,
            format!("Starting code generation for the block for scope {}", self.max_scope)
        );

        // Manually set the current scope because we are not able to look down
        // in the symbol table
        symbol_table.set_cur_scope(self.max_scope);

        // The current node is the block, so we need to loop through each of its children
        let neighbors: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();

        // Assume a success
        let mut block_res: bool = true;

        for neighbor_index in neighbors.into_iter().rev() {
            let child: &SyntaxTreeNode = (*ast).graph.node_weight(neighbor_index).unwrap();
            
            match child {
                SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                    block_res = match non_terminal {
                        NonTerminalsAst::Block => self.code_gen_block(ast, neighbor_index, symbol_table),
                        NonTerminalsAst::VarDecl => self.code_gen_var_decl(ast, neighbor_index, symbol_table),
                        NonTerminalsAst::Assign => self.code_gen_assignment(ast, neighbor_index, symbol_table),
                        NonTerminalsAst::Print => self.code_gen_print(ast, neighbor_index, symbol_table),
                        NonTerminalsAst::If => self.code_gen_if(ast, neighbor_index, symbol_table),
                        NonTerminalsAst::While => self.code_gen_while(ast, neighbor_index, symbol_table),
                        _ => { 
                            error!("Received {:?} when expecting an AST nonterminal statement in a block", non_terminal);
                            false
                        }
                    };
                    if !block_res {
                        return false;
                    }
                }
                _ => error!("Received {:?} when expecting an AST nonterminal for code gen in a block", child)
            }
        }

        // Exit the current scope
        symbol_table.end_cur_scope();
        return block_res;
    }

    fn add_print_int_code(&mut self) {
        // Function is called print_int
        self.code_arr.push(format!("print_int:"));

        // Get the byte stored in a0
        // Assume a0 is the number that needs to be printed
        self.code_arr.push(format!("mv t0, a0"));

        // Sys call 64 is printing
        self.code_arr.push(format!("li  a7, 64"));
        // a0 = 1 is sysout
        self.code_arr.push(format!("li  a0, 1"));
        // a1 is the address of the string to print
        self.code_arr.push(format!("la  a1, print_int_char"));
        // a2 is the length of the string (1 digit at a time)
        self.code_arr.push(format!("li  a2, 1"));

        // t1 is the index of the string we are on
        self.code_arr.push(format!("li  t1, 0"));

        // t2 is what we are dividing by to get the digit
        // Starts with 100 because a byte is no longer than 3 digits long in base 10
        self.code_arr.push(format!("li  t2, 100"));

        // No more than 3 iterations of the loop
        self.code_arr.push(format!("li  t3, 3"));

        // 10 has to be stored for later use
        self.code_arr.push(format!("li  t4, 10"));

        // Create the label for the loop
        self.code_arr.push(format!("print_int_loop:"));
        
        // Get the top digit
        self.code_arr.push(format!("divu  t5, t0, t2"));
        // Add 0x30 to convert from digit to ascii (0 is 0x30 - 9 is 0x39)
        self.code_arr.push(format!("addi  t5, t5, 0x30"));

        // a1 already has the address of the byte we are storing
        self.code_arr.push(format!("sb  t5, 0(a1)"));

        // Make the sys call to print the digit
        self.code_arr.push(format!("ecall"));

        // Get the remainder
        self.code_arr.push(format!("remu  t0, t0, t2"));

        // Decrease the number we are dividing by
        self.code_arr.push(format!("divu  t2, t2, t4"));

        // Increment the counter
        self.code_arr.push(format!("addi  t1, t1, 1"));

        // Branch to top of loop if still more digits to print
        self.code_arr.push(format!("blt  t1, t3, print_int_loop"));

        // Return from the function call
        self.code_arr.push(format!("ret"));
    }

    fn add_print_string_code(&mut self) {
        // Create the label for printing the string
        self.code_arr.push(format!("print_string:"));

        // Assume a0 has the address of the string to print
        self.code_arr.push(format!("mv  t0, a0"));

        // Basic setup for the sys call
        self.code_arr.push(format!("li  a7, 64"));
        self.code_arr.push(format!("li  a0, 1"));

        // The halfword is the length of the string
        self.code_arr.push(format!("lhu  a2, 0(t0)"));

        // 2 bytes over is the start of the string
        self.code_arr.push(format!("addi  a1, t0, 2"));
        self.code_arr.push(format!("ecall"));

        self.code_arr.push(format!("ret"));
    }

    fn add_print_boolean_code(&mut self) {
        self.code_arr.push(format!("print_boolean:"));

        // Assume a0 has the boolean value
        self.code_arr.push(format!("beq  a0, zero, print_false"));

        // If the var is true, load true
        self.code_arr.push(format!("la  a0, string_1"));
        self.code_arr.push(format!("j  print_bool_call"));

        self.code_arr.push(format!("print_false:"));
        // Otherwise load false
        self.code_arr.push(format!("la  a0, string_0"));

        self.code_arr.push(format!("print_bool_call:"));
        
        self.code_arr.push(format!("addi  sp, sp, -4"));
        self.code_arr.push(format!("sw  ra, 0(sp)"));

        // Print the string for the respective value of the variable
        self.code_arr.push(format!("call print_string"));

        self.code_arr.push(format!("lw  ra, 0(sp)"));
        self.code_arr.push(format!("addi  sp, sp, 4"));

        self.code_arr.push(format!("ret"));
    }

    fn add_print_new_line_code(&mut self) {
        // Create the label for a new line subroutine
        self.code_arr.push(format!("print_new_line:"));

        // Print out the new line character
        self.code_arr.push(format!("li  a7, 64"));
        self.code_arr.push(format!("li  a0, 1"));
        self.code_arr.push(format!("la  a1, new_line"));
        self.code_arr.push(format!("li  a2, 1"));
        self.code_arr.push(format!("ecall"));

        self.code_arr.push(format!("ret"));
    }

    fn add_compare_eq_code(&mut self) {
        // Create the label for comparing equality between 2 values
        self.code_arr.push(format!("compare_eq:"));

        // Assume both values are in a0 and a1
        self.code_arr.push(format!("beq  a0, a1, compare_eq_true"));

        // Result stored in a0
        self.code_arr.push(format!("li  a0, 0"));
        self.code_arr.push(format!("j  compare_eq_ret"));

        // Create the label for storing the true value
        self.code_arr.push(format!("compare_eq_true:"));
        self.code_arr.push(format!("li  a0, 1"));

        // Return form the subroutine
        self.code_arr.push(format!("compare_eq_ret:"));
        self.code_arr.push(format!("ret"));
    }

    fn add_compare_neq_code(&mut self) {
        // Create the label for comparing equality between 2 values
        self.code_arr.push(format!("compare_neq:"));

        // Assume both values are in a0 and a1
        self.code_arr.push(format!("bne  a0, a1, compare_neq_true"));

        // Result stored in a0
        self.code_arr.push(format!("li  a0, 0"));
        self.code_arr.push(format!("j  compare_neq_ret"));

        // Create the label for storing the true value
        self.code_arr.push(format!("compare_neq_true:"));
        self.code_arr.push(format!("li  a0, 1"));

        // Return form the subroutine
        self.code_arr.push(format!("compare_neq_ret:"));
        self.code_arr.push(format!("ret"));
    }

    fn create_output_string(&mut self) -> String {
        let mut output_builder: Builder = Builder::default();
        
        output_builder.append(".section .text<br>");
        output_builder.append(".global _start<br>");
        output_builder.append("_start:<br>");
        output_builder.append("nop<br>");
        for code in self.code_arr.iter() {
            output_builder.append(code.as_str());
            output_builder.append("<br>");
        }

        //output_builder.append(".section .data\n");
        for static_data in self.static_arr.iter() {
            output_builder.append(static_data.as_str());
            output_builder.append("<br>");
        }

        for heap_data in self.heap_arr.iter() {
            output_builder.append(heap_data.as_str());
            output_builder.append("<br>");
        }

        return output_builder.string().unwrap();
    }

    fn store_string(&mut self, string: &str) -> usize {
        let addr: Option<&usize> = self.string_history.get(string);
        if addr.is_none() {
            // Place the string in the heap
            self.heap_arr.push(format!("string_{}:", self.string_history.len()));
            // We will let strings be no longer than 2^16 - 1
            self.heap_arr.push(format!(".half {}", string.len()));
            self.heap_arr.push(format!(".ascii \"{}\"", string));
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::CodeGenerator,
                format!("Stored string \"{}\" at label string_{}", string, self.string_history.len())
            );

            // Store it for future use
            self.string_history.insert(String::from(string), self.string_history.len());

            // Since it has been stored, we need to return 1 minus the index
            return self.string_history.len() - 1;
        } else {
            // The string is already on the heap, so return its address
            return *addr.unwrap();
        }
    }

    // Function for creating the code for a variable declaration
    fn code_gen_var_decl(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) -> bool {
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::CodeGenerator,
            format!("Starting code generation for variable declaration statement in scope {}", symbol_table.cur_scope.unwrap())
        );

        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
        let id_node: &SyntaxTreeNode = (*ast).graph.node_weight(children[0]).unwrap();

        match id_node {
            SyntaxTreeNode::Terminal(token) => {
                // Get the symbol table entry to get the type of the variable
                let symbol_table_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap();
                match symbol_table_entry.symbol_type {
                    // Only integers and booleans are initialized
                    Type::Int | Type::Boolean => {
                        self.static_arr.push(format!("{}_{}: .byte 0", token.text, symbol_table_entry.scope));
                        // Generate the code for the variable initialization to 1
                        self.code_arr.push(format!("la  t1, {}_{}", token.text, symbol_table_entry.scope));
                        self.code_arr.push(format!("li  t0, 0"));
                        self.code_arr.push(format!("sb  t0, 0(t1)"));
                    },
                    // Strings do not get initialized
                    Type::String => {
                        // Only have to create the static entry here
                        // Since it is a string on the heap, we have to store the address
                        // which is a full word
                        self.static_arr.push(format!("{}_{}: .word 0", token.text, symbol_table_entry.scope));
                    }
                }
            },
            _ => error!("Received {:?} when expecting terminal for var decl child in code gen", id_node)
        }

        return true;
    }

    // Function for creating the code for an assignment
    fn code_gen_assignment(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) -> bool {
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::CodeGenerator,
            format!("Starting code generation for assignment statement in scope {}", symbol_table.cur_scope.unwrap())
        );

        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
        let value_node: &SyntaxTreeNode = (*ast).graph.node_weight(children[0]).unwrap();
        let id_node: &SyntaxTreeNode = (*ast).graph.node_weight(children[1]).unwrap();

        match value_node {
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Identifier(id_name) => {
                        let value_id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 
                        
                        // Load the address of the value variable then load the data
                        self.code_arr.push(format!("la  t2, {}_{}", id_name, value_id_entry.scope));

                        match value_id_entry.symbol_type {
                            Type::Int | Type::Boolean => {
                                // Load only a byte for integers and booleans
                                self.code_arr.push(format!("lbu t0, 0(t2)"));
                            },
                            Type::String => {
                                // Strings are an entire word
                                self.code_arr.push(format!("lwu t0, 0(t2)"));
                            }
                        }
                    },
                    TokenType::Digit(val) => {
                        // Digits just load a constant to the accumulator
                        self.code_arr.push(format!("li  t0, {}", val)); 
                    },
                    TokenType::Char(string) => {
                        // Start by storing the string
                        let string_index: usize = self.store_string(&string);

                        // Store the starting address of the string in memory
                        self.code_arr.push(format!("la  t0, string_{}", string_index));
                    },
                    TokenType::Keyword(keyword) => {
                        match &keyword {
                            Keywords::True => {
                                // True is 1
                                self.code_arr.push(format!("li  t0, 1"));
                            },
                            Keywords::False => {
                                // False is 0
                                self.code_arr.push(format!("li  t0, 0")); 
                            },
                            _ => error!("Received {:?} when expecting true or false for keyword terminals in assignment", keyword)
                        }
                    },
                    _ => error!("Received {:?} for terminal in assignment when expecting id, digit, char, or keyword", token)
                }
            },
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                match non_terminal {
                    NonTerminalsAst::Add => {
                        // Call add, so the result will be in both the accumulator and in memory
                        if !self.code_gen_add(ast, children[0], symbol_table, true) { return false; }
                    },
                    NonTerminalsAst::IsEq => {
                        if !self.code_gen_compare(ast, children[0], symbol_table, true) { return false; }
                        self.code_arr.push(format!("mv  t0, a0"));
                    },
                    NonTerminalsAst::NotEq => {
                        if !self.code_gen_compare(ast, children[0], symbol_table, false) { return false; }
                        self.code_arr.push(format!("mv  t0, a0"));
                    },
                    _ => error!("Received {:?} for nonterminal on right side of assignment for code gen", non_terminal)
                }
            },
            _ => error!("Received {:?} when expecting terminal or AST nonterminal for assignment in code gen", value_node)
        }

        match id_node {
            SyntaxTreeNode::Terminal(token) => {
                // Get the static offset for the variable being assigned to
                let id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 
                
                // The data that we are storing is already in t0, so load the appropriate
                // address and store the data

                self.code_arr.push(format!("la  t1, {}_{}", token.text, id_entry.scope));
                match &id_entry.symbol_type {
                    Type::Int | Type::Boolean => {
                        // Int and boolean take up only 1 byte
                        self.code_arr.push(format!("sb  t0, 0(t1)")); 
                    },
                    Type::String => {
                        // Strings take up a full word
                        self.code_arr.push(format!("sw  t0, 0(t1)"));
                    }
                }
            },
            _ => error!("Received {:?} when expecting terminal for assignmentchild in code gen", id_node)
        }

        return true;
    }

    // Function for generating code for a print statement
    fn code_gen_print(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) -> bool {
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::CodeGenerator,
            format!("Starting code generation for print statement in scope {}", symbol_table.cur_scope.unwrap())
        );

        // Get the child on the print statement to evaluate
        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
        let child: &SyntaxTreeNode = (*ast).graph.node_weight(children[0]).unwrap();

        match child {
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Identifier(id_name) => {
                        let print_id: &SymbolTableEntry = symbol_table.get_symbol(&id_name).unwrap();
                        match &print_id.symbol_type {
                            Type::Int => {
                                self.code_arr.push(format!("la  t0, {}_{}", id_name, print_id.scope));
                                self.code_arr.push(format!("lbu  a0, 0(t0)"));
                                self.code_arr.push(format!("call print_int"));
                            },
                            Type::String => {
                                // Store the string address in Y
                                self.code_arr.push(format!("lwu  a0, {}_{}", id_name, print_id.scope));
                                self.code_arr.push(format!("call print_string"));
                            },
                            Type::Boolean => {
                                // Compare the value of the variable with false
                                self.code_arr.push(format!("lbu  a0, {}_{}", id_name, print_id.scope));
                                self.code_arr.push(format!("call print_boolean"));
                            }
                        }
                    },
                    TokenType::Digit(digit) => {
                        // Place the number in a0 and call the function that
                        // handles numbers
                        self.code_arr.push(format!("li  a0, {}", digit));
                        self.code_arr.push(format!("call print_int"));
                    },
                    TokenType::Char(string) => {
                        // Store the string in memory and get its index
                        let string_index: usize = self.store_string(&string);

                        // Get the address of the string we want to print
                        self.code_arr.push(format!("la  a0, string_{}", string_index));
                        self.code_arr.push(format!("call print_string"));
                    },
                    TokenType::Keyword(keyword) => {
                        match keyword {
                            Keywords::True => {
                                // Load the address for true
                                self.code_arr.push(format!("la  a0, string_1"));
                            },
                            Keywords::False => {
                                // Load the address for false
                                self.code_arr.push(format!("la  a0, string_0"));
                            },
                            _ => error!("Received {:?} when expecting true or false for print keyword", keyword)
                        }
                        // Make the system call
                        self.code_arr.push(format!("call print_string"));
                    },
                    _ => error!("Received {:?} when expecting id, digit, string, or keyword for print terminal", token)
                }
            },
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                match non_terminal {
                    NonTerminalsAst::Add => {
                        // Generate the result of the addition expression
                        if !self.code_gen_add(ast, children[0], symbol_table, true) { return false; }
                        
                        // Move the contents in t0 to a0
                        self.code_arr.push(format!("mv  a0, t0"));
                        self.code_arr.push(format!("call print_int")); 
                    },
                    NonTerminalsAst::IsEq => {
                        // The result of the equality comparison is in a0
                        self.code_gen_compare(ast, children[0], symbol_table, true);
                        self.code_arr.push(format!("call print_boolean"));
                    },
                    NonTerminalsAst::NotEq => {
                        self.code_gen_compare(ast, children[0], symbol_table, false);
                        self.code_arr.push(format!("call print_boolean"));
                    },
                    _ => error!("Received {:?} when expecting addition or boolean expression for nonterminal print", non_terminal)
                }
            },
            _ => error!("Received {:?} when expecting terminal or AST nonterminal for print in code gen", child)
        }

        // Add a new line for cleanliness
        self.code_arr.push(format!("call print_new_line"));

        return true;
    }

    // Function to generate code for an addition statement
    // Result is left in t0
    fn code_gen_add(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable, is_first: bool) -> bool {
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::CodeGenerator,
            format!("Starting code generation for addition expression in scope {}", symbol_table.cur_scope.unwrap())
        );

        // Get the child for addition
        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
        let right_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[0]).unwrap();
        let left_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[1]).unwrap();

        match right_child {
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Digit(num) => {
                        // Store right side digit in t0
                        self.code_arr.push(format!("li  t1, {}", num));
                    },
                    TokenType::Identifier(id_name) => {
                        // Get the address needed from memory for the identifier
                        let value_id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 
                        
                        // Load the variable's value into t0
                        self.code_arr.push(format!("la  t2, {}_{}", id_name, value_id_entry.scope));
                        self.code_arr.push(format!("lbu  t1, 0(t2)"));
                    },
                    _ => error!("Received {:?} when expecting digit or id for right side of addition", token)
                }
            },
            // Nonterminals are always add, so just call it
            SyntaxTreeNode::NonTerminalAst(_) => if !self.code_gen_add(ast, children[0], symbol_table, false) { return false; },
            _ => error!("Received {:?} when expecting terminal or AST nonterminal for right addition value", right_child)
        }

        match left_child {
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Digit(num) => {
                        // Load the number to t0
                        self.code_arr.push(format!("li  t0, {}", num));
                        if is_first {
                            // If we are in the outermost add, then store the
                            // result in t0
                            self.code_arr.push(format!("add  t0, t0, t1"));
                        } else {
                            // Otherwise store it in t1 because there are still
                            // more elements to add that will be loaded into t0
                            self.code_arr.push(format!("add  t1, t0, t1"));
                        }
                    },
                    _ => error!("Received {:?} when expecting a digit for left side of addition for code gen", token)
                }
            },
            _ => error!("Received {:?} when expecting a terminal for the left side of addition for code gen", left_child)
        }

        return true;
    }

    // Function to generate code for comparisons
    // Result is left in the Z flag and get_z_flag_vale function can be used
    // afterwards to place z flag value into the accumulator
    fn code_gen_compare(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable, is_eq: bool) -> bool {
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::CodeGenerator,
            format!("Starting code generation for comparison expression (is_eq = {}) in scope {}", is_eq, symbol_table.cur_scope.unwrap())
        );

        // Get the child for comparison
        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
        let right_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[0]).unwrap();
        let left_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[1]).unwrap();

        match left_child {
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Identifier(id_name) => {
                        // Get the address needed from memory for the identifier
                        let value_id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 
                        
                        // Get the address of the variable
                        self.code_arr.push(format!("la  t0, {}_{}", id_name, value_id_entry.scope));

                        // Now store the value of the variable in a0
                        match value_id_entry.symbol_type {
                            Type::Int | Type::Boolean => {
                                self.code_arr.push(format!("lbu  a0, 0(t0)"));
                            },
                            Type::String => {
                                self.code_arr.push(format!("lwu  a0, 0(t0)"));
                            }
                        }
                    },
                    TokenType::Digit(num) => {
                        // Store the digit in a0
                        self.code_arr.push(format!("li  a0, {}", num));
                    },
                    TokenType::Char(string) => {
                        // Store the address of the string in a0
                        let string_index: usize = self.store_string(string);
                        self.code_arr.push(format!("la  a0, string_{}", string_index));
                    },
                    TokenType::Keyword(keyword) => {
                        match &keyword {
                            Keywords::True => self.code_arr.push(format!("li  a0, 1")),
                            Keywords::False => self.code_arr.push(format!("li  a0, 0")),
                            _ => error!("Received {:?} when expecting true or false for keywords in boolean expression", keyword)
                        }
                    },
                    _ => error!("Received {:?} when expecting an Id, digit, char, or keyword for left side of boolean expression", token)
                }
            },
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                match &non_terminal {
                    NonTerminalsAst::Add => {
                        // Run the addition and move the result from t0 to a0
                        self.code_gen_add(ast, children[1], symbol_table, true);
                        self.code_arr.push(format!("mv  a0, t0"));
                    },
                    NonTerminalsAst::IsEq => {
                        if !self.code_gen_compare(ast, children[1], symbol_table, true) { return false; }
                    },
                    NonTerminalsAst::NotEq => {
                        if !self.code_gen_compare(ast, children[1], symbol_table, false) { return false; }
                    },
                    _ => error!("Received {:?} for left side of nonterminal boolean expression, when expected Add, IsEq, or NotEq", non_terminal)
                }
            },
            _ => error!("Received {:?} when expected terminal or AST nonterminal for left side of comparison in code gen", left_child)
        }

        match right_child {
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Identifier(id_name) => {
                        // Get the address needed from memory for the identifier
                        let value_id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 

                        // Get the address of the variable
                        self.code_arr.push(format!("la  t0, {}_{}", id_name, value_id_entry.scope));

                        // Now store the value of the variable in a1
                        match value_id_entry.symbol_type {
                            Type::Int | Type::Boolean => {
                                self.code_arr.push(format!("lbu  a1, 0(t0)"));
                            },
                            Type::String => {
                                self.code_arr.push(format!("lwu  a1, 0(t0)"));
                            }
                        }
                    },
                    TokenType::Digit(num) => {
                        // Store the digit in a1
                        self.code_arr.push(format!("li  a1, {}", num));
                    },
                    TokenType::Char(string) => {
                        // Store the address of the string in a1
                        let string_index: usize = self.store_string(string);
                        self.code_arr.push(format!("la  a1, string_{}", string_index));
                    },
                    TokenType::Keyword(keyword) => {
                        match &keyword {
                            Keywords::True => self.code_arr.push(format!("li  a1, 1")),
                            Keywords::False => self.code_arr.push(format!("li  a1, 0")),
                            _ => error!("Received {:?} when expecting true or false for keywords in boolean expression", keyword)
                        }
                    },
                    _ => error!("Received {:?} when expecting an Id, digit, char, or keyword for left side of boolean expression", token)
                }
            },
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                // We have a nonterminal, so store the left side on the stack so there is no
                // conflict with the right side evaluation
                self.code_arr.push(format!("addi  sp, sp, -1"));
                self.code_arr.push(format!("sb  a0, 0(sp)"));

                match &non_terminal {
                    NonTerminalsAst::Add => {
                        // Do the add and move the result from t0 to a1
                        self.code_gen_add(ast, children[0], symbol_table, true);
                        self.code_arr.push(format!("mv  a1, t0"));
                    },
                    NonTerminalsAst::IsEq => {
                        // Move the result over to a1
                        self.code_gen_compare(ast, children[0], symbol_table, true);
                        self.code_arr.push(format!("mv  a1, a0"));
                    },
                    NonTerminalsAst::NotEq => {
                        self.code_gen_compare(ast, children[0], symbol_table, false);
                        self.code_arr.push(format!("mv  a1, a0"));
                    },
                    _ => error!("Received {:?} for right side of nonterminal boolean expression, when expected Add, IsEq, or NotEq", non_terminal)
                }

                // Get the left side back to a0
                self.code_arr.push(format!("lbu  a0, 0(sp)"));
                self.code_arr.push(format!("addi  sp, sp, 1"));
            },
            _ => error!("Received {:?} when expected terminal or AST nonterminal for left side of comparison in code gen", left_child)
        }

        // Perform the appropriate comparison
        if is_eq {
            self.code_arr.push(format!("call compare_eq"));
        } else {
            self.code_arr.push(format!("call compare_neq"));
        }

        return true;
    }

    fn code_gen_if(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) -> bool {
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::CodeGenerator,
            format!("Starting code generation for if statement in scope {}", symbol_table.cur_scope.unwrap())
        );

        // Get the child for comparison
        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
        let left_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[1]).unwrap();

        // Get the index of the current if statement
        let if_index: usize = self.if_count.to_owned();

        match left_child {
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                match &non_terminal {
                    // Evaluate the boolean expression for the if statement
                    NonTerminalsAst::IsEq => if !self.code_gen_compare(ast, children[1], symbol_table, true) { return false; },
                    NonTerminalsAst::NotEq => if !self.code_gen_compare(ast, children[1], symbol_table, false) { return false; },
                    _ => error!("Received {:?} when expecting IsEq or NotEq for nonterminal if expression", non_terminal)
                }
                // Add the branch code
                self.code_arr.push(format!("beq  a0, zero, if_end_{}", if_index)); 
                self.if_count += 1;
            },
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Keyword(Keywords::True) => { /* Small optimization because no comparison is needed */ }
                    TokenType::Keyword(Keywords::False) => {
                        // No code should be generated here because the if-statement is just dead
                        // code and will never be reached, so no point in trying to store the code
                        // with the limited space that we already have (256 bytes)
                        return true;
                    }
                    _ => error!("Received {:?} when expecting true or false for if expression terminals", token)
                }
            },
            _ => error!("Received {:?} when expecting AST nonterminal or a terminal", left_child)
        }

        // Generate the code for the body
        if !self.code_gen_block(ast, children[0], symbol_table) { return false; }

        // Only add the label if it is needed
        if if_index != self.if_count {
            // Add the label for the end of the if statement
            self.code_arr.push(format!("if_end_{}:", if_index));
        }

        return true;
    }

    fn code_gen_while(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) -> bool {
         nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::CodeGenerator,
            format!("Starting code generation for while statement in scope {}", symbol_table.cur_scope.unwrap())
        );

        // Get the child for comparison
        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
        let left_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[1]).unwrap();

        // Get the index of the current start
        let while_index: usize = self.while_count.to_owned();
        self.while_count += 1;

        self.code_arr.push(format!("while_start_{}:", while_index));

        match left_child {
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                match &non_terminal {
                    // Evaluate the boolean expression for the while statement
                    // The Z flag is set by these function calls
                    NonTerminalsAst::IsEq => if !self.code_gen_compare(ast, children[1], symbol_table, true) { return false; },
                    NonTerminalsAst::NotEq => if !self.code_gen_compare(ast, children[1], symbol_table, false) { return false; },
                    _ => error!("Received {:?} when expecting IsEq or NotEq for nonterminal if expression", non_terminal)
                }
                // Add the branch code
                self.code_arr.push(format!("beq  a0, zero, while_end_{}", while_index));
            },
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Keyword(Keywords::True) => { /* Small optimization because no comparison is needed */ }
                    TokenType::Keyword(Keywords::False) => {
                        // No code should be generated here because the while-statement is just dead
                        // code and will never be reached, so no point in trying to store the code
                        // with the limited space that we already have (256 bytes)
                        return true;
                    }
                    _ => error!("Received {:?} when expecting true or false for while expression terminals", token)
                }
            },
            _ => error!("Received {:?} when expecting AST nonterminal or a terminal", left_child)
        }

        // Generate the code for the body
        if !self.code_gen_block(ast, children[0], symbol_table) { return false; }

        // Jump back to the condition
        self.code_arr.push(format!("j  while_start_{}", while_index));

        // Label for the end of the while block
        self.code_arr.push(format!("while_end_{}:", while_index));

        return true;
    }

    fn display_code(&mut self, program_number: &u32) {
        let window: Window = web_sys::window().expect("Should be able to get the window");
        let document: Document = window.document().expect("Should be able to get the document");

        let code_gen_tabs: Element = document.get_element_by_id("code-gen-tabs").expect("Should be able to get the element");

        // Create the new tab in the list
        let new_li: Element = document.create_element("li").expect("Should be able to create the li element");

        // Add the appropriate classes
        let li_classes: DomTokenList = new_li.class_list();
        li_classes.add_1("nav-item").expect("Should be able to add the class");
        new_li.set_attribute("role", "presentation").expect("Should be able to add the attribute");

        // Create the button
        let new_button: Element = document.create_element("button").expect("Should be able to create the button");
        let btn_classes: DomTokenList = new_button.class_list();
        btn_classes.add_1("nav-link").expect("Should be able to add the class");

        // Only make the first one active
        if code_gen_tabs.child_element_count() == 0 {
            btn_classes.add_1("active").expect("Should be able to add the class");
            new_button.set_attribute("aria-selected", "true").expect("Should be able to add the attribute");
        } else {
            new_button.set_attribute("aria-selected", "false").expect("Should be able to add the attribute");
        }

        // Set the id of the button
        new_button.set_id(format!("program{}-code-gen-btn", *program_number).as_str());

        // All of the toggle elements from the example above
        new_button.set_attribute("data-bs-toggle", "tab").expect("Should be able to add the attribute");
        new_button.set_attribute("type", "button").expect("Should be able to add the attribute");
        new_button.set_attribute("role", "tab").expect("Should be able to add the attribute");
        new_button.set_attribute("data-bs-target", format!("#program{}-code-gen-pane", *program_number).as_str()).expect("Should be able to add the attribute");
        new_button.set_attribute("aria-controls", format!("program{}-code-gen-pane", *program_number).as_str()).expect("Should be able to add the attribute");

        // Set the inner text
        new_button.set_inner_html(format!("Program {}", *program_number).as_str());

        // Append the button and the list element to the area
        new_li.append_child(&new_button).expect("Should be able to add the child node");
        code_gen_tabs.append_child(&new_li).expect("Should be able to add the child node");

        // Get the content area
        let content_area: Element = document.get_element_by_id("code-gen-tab-content").expect("Should be able to find the element");

        // Create the individual pane div
        let display_area_div: Element = document.create_element("div").expect("Should be able to create the element");

        // Also from the example link above to only let the first pane initially show and be active
        let display_area_class_list: DomTokenList = display_area_div.class_list();
        display_area_class_list.add_1("tab-pane").expect("Should be able to add the class");
        if content_area.child_element_count() == 0 {
            display_area_class_list.add_2("show", "active").expect("Should be able to add the classes");
        }

        // Add the appropriate attributes
        display_area_div.set_attribute("role", "tabpanel").expect("Should be able to add the attribute");
        display_area_div.set_attribute("tabindex", "0").expect("Should be able to add the attribute");
        display_area_div.set_attribute("aria-labeledby", format!("program{}-code-gen-btn", *program_number).as_str()).expect("Should be able to add the attribute");

        // Set the id of the pane
        display_area_div.set_id(format!("program{}-code-gen-pane", *program_number).as_str());

        // The div is a container for the content of the ast info
        display_area_class_list.add_3("container", "text-center", "code-gen-pane").expect("Should be able to add the classes");

        // Generate the final assembly output string
        let mut code_str: String = self.create_output_string();

        // This is the element that the code is in
        let code_elem: Element = document.create_element("p").expect("Should be able to create the element");
        let code_elem_class_list: DomTokenList = code_elem.class_list();
        code_elem_class_list.add_2("overflow-auto", "code-text").expect("Should be able to add the classes");
//        code_elem.set_class_name("code-text");
        code_elem.set_inner_html(&code_str);

        let code_str_clipboard: String = code_str.as_str().replace("<br>", "\n");

        display_area_div.append_child(&code_elem).expect("Should be able to add the child node");

        // This is the button to copy to the clipboard
        let copy_btn: Element = document.create_element("button").expect("Should be able to create the element");
        copy_btn.set_inner_html("Copy to Clipboard");
        copy_btn.set_class_name("copy-btn");
        display_area_div.append_child(&copy_btn).expect("Should be able to add the child node");

        // Create a function that will be used as the event listener and add it to the copy button
        let copy_btn_fn: Closure<dyn FnMut()> = Closure::wrap(Box::new(move || {
            // Call the JS function that handles the clipboard
            set_clipboard(&code_str_clipboard);
        }) as Box<dyn FnMut()>);
        copy_btn.add_event_listener_with_callback("click", copy_btn_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
        copy_btn_fn.forget();

        // Add the div to the pane
        content_area.append_child(&display_area_div).expect("Should be able to add the child node");
    }

//    pub fn clear_display() {
//        // Get the preliminary objects
//        let window: Window = web_sys::window().expect("Should be able to get the window");
//        let document: Document = window.document().expect("Should be able to get the document");
//
//        // Clear the entire area
//        let tabs_area: Element = document.get_element_by_id("code-gen-tabs").expect("Should be able to find the element");
//        tabs_area.set_inner_html("");
//        let content_area: Element = document.get_element_by_id("code-gen-tab-content").expect("Should be able to find the element");
//        content_area.set_inner_html("");
//    }
}
