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

// Have to import the editor js module
#[wasm_bindgen(module = "/editor.js")]
extern "C" {
    // Import the getCodeInput function from js so we can call it from the Rust code
    #[wasm_bindgen(js_name = "setClipboard")]
    fn set_clipboard(newText: &str);
}

enum CodeGenBytes {
    // Representation for final code/data in memory
    Code(u8),
    // Temporary variable address  until AST is traversed with identifier for later use
    Var(usize),
    // Temproary data for addition and boolean expression evaluation
    Temp(usize),
    // Spot is available for anything to take it
    Empty,
    // Represents data on the heap
    Data(u8),
    // This is a jump address for if and while statements
    Jump(usize),
    // This is the unknown high order byte for var and temp data
    HighOrderByte,
}

// Customize the output when printing the string
impl fmt::Debug for CodeGenBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CodeGenBytes::Code(code) => write!(f, "{:02X}", code),
            CodeGenBytes::Var(var) => write!(f, "V{}", var),
            CodeGenBytes::Temp(temp) => write!(f, "T{}", temp),
            CodeGenBytes::Empty => write!(f, "00"),
            CodeGenBytes::Data(data) => write!(f, "{:02X}", data),
            CodeGenBytes::Jump(jump) => write!(f, "J{}", jump),
            CodeGenBytes::HighOrderByte => write!(f, "XX")
        }
    }
}

// The struct for the code generator
#[derive (Debug)]
pub struct CodeGenerator {
    // The current max scope we have seen so far, which are encountered in
    // sequential order
    max_scope: usize,
    
    // The array for code gen
    code_arr: Vec<CodeGenBytes>,

    // The current location of the code in the memory array
    // The stack pointer is always code_pointer + 1
    code_pointer: u8,

    // The current location of the heap from the back of the array
    heap_pointer: u8,

    // The static table hashmap for <(id, scope), offset>
    static_table: HashMap<(String, usize), usize>,

    // Index for the temoprary data
    temp_index: usize,

    // Hashmap to keep track of the strings being stored on the heap
    string_history: HashMap<String, u8>,

    // Vector to keep track of each jump in the code
    jumps: Vec<u8>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        let mut code_gen: CodeGenerator = CodeGenerator {
            // This is a flag for a new program
            max_scope: usize::MAX,

            // We are only able to store 256 bytes in memory
            code_arr: Vec::with_capacity(0x100),

            // Code starts at 0x00
            code_pointer: 0x00,

            // Heap starts at 0xFE (0xFF reserved for 0x00)
            heap_pointer: 0xFE,

            static_table: HashMap::new(),

            // Always start with a temp index of 0
            temp_index: 0,

            string_history: HashMap::new(),

            jumps: Vec::new()
        };

        // Initialize the entire array to be unused spot in memory
        for _ in 0..0x100 {
            code_gen.code_arr.push(CodeGenBytes::Empty);
        }

        return code_gen;
    }

    pub fn generate_code(&mut self, ast: &SyntaxTree, symbol_table: &mut SymbolTable, program_number: &u32) {
        // Make sure the current scope is set to be a flag for none
        self.max_scope = usize::MAX;
        
        // Reset the array and empty it out
        for i in 0..0x100 {
            self.code_arr[i] = CodeGenBytes::Empty;
        }

        self.code_pointer = 0x00;
        self.heap_pointer = 0xFE;

        self.static_table.clear();
        self.temp_index = 0;
        self.string_history.clear();
        self.jumps.clear();

        // We are going to store the strings false and true to print them
        // out instead of 0 and 1
        self.store_string("false");
        self.store_string("true");

        // Generate the code for the program
        let program_res: bool = self.code_gen_block(ast, NodeIndex::new((*ast).root.unwrap()), symbol_table);
        debug!("{:?}", self.code_arr);

        if program_res {
            // All programs end with 0x00, which is HALT
            let final_res: bool = self.add_code(0x00);
            debug!("{:?}", self.code_arr);

            if final_res {
                self.backpatch_addresses();

                debug!("Static table: {:?}", self.static_table);
                debug!("Jumps vector: {:?}", self.jumps);
                debug!("{:?}", self.code_arr);

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
                return;
            }
        }

        nexus_log::log(
            nexus_log::LogTypes::Error,
            nexus_log::LogSources::CodeGenerator,
            format!("Code generation failed")
        );
        
        nexus_log::insert_empty_line();

        nexus_log::log(
            nexus_log::LogTypes::Warning,
            nexus_log::LogSources::Nexus,
            format!("Executable image display skipped due to code generation failure")
        );
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

    fn has_available_memory(&mut self) -> bool {
        let num_vars: usize = self.static_table.len();
        // Check for collision at the double bar (where stack meets heap)
        //  |  Code  |  Vars  ||  Temp  |  Heap  |
        return self.code_pointer + (num_vars as u8) <= self.heap_pointer - (self.temp_index as u8);
    }

    // Function to add byte of code to the memory array
    fn add_code(&mut self, code: u8) -> bool {
        if self.has_available_memory() {
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::CodeGenerator,
                format!("Adding code 0x{:02X} at memory location 0x{:02X}", code, self.code_pointer)
            );

            // Add the code to the next available spot in memory
            self.code_arr[self.code_pointer as usize] = CodeGenBytes::Code(code);
            self.code_pointer += 1;
            // No error, so successful addition to the code
            return true;
        } else {
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::CodeGenerator,
                String::from("The stack has collided with the heap causing a stack overflow error")
            );
            return false;
        }
    }

    // Function to add byte of code to the memory array for variable addressing
    fn add_var(&mut self, var: usize) -> bool {
        if self.has_available_memory() {
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::CodeGenerator,
                format!("Adding variable placeholder {} at memory location 0x{:02X}", var, self.code_pointer)
            );

            // Add the code to the next available spot in memory
            self.code_arr[self.code_pointer as usize] = CodeGenBytes::Var(var);
            self.code_pointer += 1;
            // All vars are followed by the high order byte
            return self.add_high_order_byte();
        } else {
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::CodeGenerator,
                String::from("The stack has collided with the heap causing a stack overflow error")
            );
            return false;
        }
    }

    // Function to add the high order byte for unknown addresses that will be backpatched
    fn add_high_order_byte(&mut self) -> bool {
        if self.has_available_memory() {
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::CodeGenerator,
                format!("Adding high order byte placeholder at memory location 0x{:02X}", self.code_pointer)
            );

            // Add the code to the next available spot in memory
            self.code_arr[self.code_pointer as usize] = CodeGenBytes::HighOrderByte;
            self.code_pointer += 1;
            return true;
        } else {
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::CodeGenerator,
                String::from("The stack has collided with the heap causing a stack overflow error")
            );
            return false;
        }
    }

    // Function to create space for new temp data and return its index
    fn new_temp(&mut self) -> Option<usize> {
        if self.has_available_memory() {
            // Make the room for the single byte
            let temp_addr: usize = self.temp_index.to_owned();
            self.temp_index += 1;
            return Some(temp_addr);
        } else {
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::CodeGenerator,
                String::from("The heap has collided with the stack causing a heap overflow error")
            );
            return None;
        }
    }

    // Function to add byte of code to memory array for temporary data
    fn add_temp(&mut self, temp: usize) -> bool {
        if self.has_available_memory() {
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::CodeGenerator,
                format!("Adding temp data placeholder {} at memory location 0x{:02X}", temp, self.code_pointer)
            );

            // Add the addressing for the temporary value
            self.code_arr[self.code_pointer as usize] = CodeGenBytes::Temp(temp);
            self.code_pointer += 1;
            // All temps are followed by the high order byte
            return self.add_high_order_byte();
        } else {
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::CodeGenerator,
                String::from("The heap has collided with the stack causing a heap overflow error")
            );
            return false;
        }
    }

    // Function to add a byte of data to the heap
    fn add_data(&mut self, data: u8) -> bool {
        if self.has_available_memory() {
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::CodeGenerator,
                format!("Adding data 0x{:02X} at memory location 0x{:02X}", data, self.heap_pointer)
            );

            // Heap starts from the end of the 256 bytes and moves towards the front
            self.code_arr[self.heap_pointer as usize] = CodeGenBytes::Data(data);
            self.heap_pointer -= 1;
            return true;
        } else {
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::CodeGenerator,
                String::from("The heap has collided with the stack causing a heap overflow error")
            );
            return false;
        }
    }

    fn store_string(&mut self, string: &str) -> Option<u8> {
        let addr: Option<&u8> = self.string_history.get(string);
        if addr.is_none() {
            // Assume the string gets stored
            let mut is_stored: bool = true;

            // All strings are null terminated, so start with a 0x00 at the end
            self.add_data(0x00);

            // Loop through the string in reverse order
            for c in string.chars().rev() {
                // Add the ascii code of each character
                if !self.add_data(c as u8) {
                    is_stored = false;
                    // Break if there was a heap overflow error
                    break;
                }
            }
           
            if is_stored {
                nexus_log::log(
                    nexus_log::LogTypes::Debug,
                    nexus_log::LogSources::CodeGenerator,
                    format!("Stored string \"{}\" at memory location 0x{:02X}", string, self.heap_pointer + 1)
                );

                // Store it for future use
                self.string_history.insert(String::from(string), self.heap_pointer + 1);
                return Some(self.heap_pointer + 1);
            } else {
                // There is no address to return
                return None;
            }
        } else {
            // The string is already on the heap, so return its address
            return Some(*addr.unwrap());
        }
    }

    fn add_jump(&mut self) -> bool {
        if self.has_available_memory() {
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::CodeGenerator,
                format!("Adding jump placeholder {} at memory location 0x{:02X}", self.jumps.len(), self.code_pointer)
            );

            // Add the jump to the code and set it to 0 in the vector of jumps
            self.code_arr[self.code_pointer as usize] = CodeGenBytes::Jump(self.jumps.len());
            self.code_pointer += 1;
            self.jumps.push(0x00);
            return true;
        } else {
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::CodeGenerator,
                String::from("The stack has collided with the heap causing a stack overflow error")
            );
            return false;
        }
    }

    // Replaces temp addresses with the actual position in memory
    // Do not have to worry about memory availability because that was taken
    // care of when the placeholders were created
    fn backpatch_addresses(&mut self) { 
        for i in 0..self.code_arr.len() {
            match &self.code_arr[i] {
                CodeGenBytes::Var(offset) => {
                    // Compute the new address
                    let new_addr: u8 = self.code_pointer + *offset as u8;
                    nexus_log::log(
                        nexus_log::LogTypes::Debug,
                        nexus_log::LogSources::CodeGenerator,
                        format!("Backpatching 0x{:02X} for variable placeholder {} at memory location 0x{:02X}", new_addr, offset, i)
                    );

                    self.code_arr[i] = CodeGenBytes::Code(new_addr);

                    // The integer division result is the high order byte
                    // Always 0 in this case
                    let new_high: u8 = (new_addr as u16 / 0x100) as u8;

                    nexus_log::log(
                        nexus_log::LogTypes::Debug,
                        nexus_log::LogSources::CodeGenerator,
                        format!("Backpatching 0x{:02X} for high order byte placeholder at memory location 0x{:02X}", new_high, i + 1)
                    );

                    self.code_arr[i + 1] = CodeGenBytes::Code(new_high);
                },
                CodeGenBytes::Temp(offset) => {
                    // Compute the address of the temp data
                    let new_addr: u8 = self.heap_pointer - *offset as u8;
                    
                    nexus_log::log(
                        nexus_log::LogTypes::Debug,
                        nexus_log::LogSources::CodeGenerator,
                        format!("Backpatching 0x{:02X} for temp data placeholder {} at memory location 0x{:02X}", new_addr, offset, i)
                    );

                    self.code_arr[i] = CodeGenBytes::Code(new_addr);
                   
                    // The integer division result is the high order byte
                    // Always 0 in this case
                    let new_high: u8 = (new_addr as u16 / 0x100) as u8;

                    nexus_log::log(
                        nexus_log::LogTypes::Debug,
                        nexus_log::LogSources::CodeGenerator,
                        format!("Backpatching 0x{:02X} for high order byte placeholder at memory location 0x{:02X}", new_high, i + 1)
                    );

                    self.code_arr[i + 1] = CodeGenBytes::Code(new_high);
                },
                // Store the value from the jump into the placeholder
                CodeGenBytes::Jump(jump_index) => {
                    nexus_log::log(
                        nexus_log::LogTypes::Debug,
                        nexus_log::LogSources::CodeGenerator,
                        format!("Backpatching 0x{:02X} for jump placeholder {} at memory location 0x{:02X}", 
                                self.jumps[*jump_index], *jump_index, i)
                    );
                    self.code_arr[i] = CodeGenBytes::Code(self.jumps[*jump_index])
                },
                _ => {} 
            }
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
                // Get the offset this variable will be on the stack
                let static_offset: usize = self.static_table.len();
                self.static_table.insert((token.text.to_owned(), symbol_table.cur_scope.unwrap()), static_offset);

                // Get the symbol table entry to get the type of the variable
                let symbol_table_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap();
                match symbol_table_entry.symbol_type {
                    // Only integers and booleans are initialized
                    Type::Int | Type::Boolean => {
                        // Generate the code for the variable declaration
                        if !self.add_code(0xA9) { return false; }
                        if !self.add_code(0x00) { return false; }
                        if !self.add_code(0x8D) { return false; }
                        if !self.add_var(static_offset) { return false; }
                    },
                    // Strings do not get initialized
                    Type::String => {
                        // Nothing to do here, so may end up initially with dirty data
                        // from temp values
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
                    TokenType::Identifier(_) => {
                        let value_id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 
                        let value_static_offset: usize = self.static_table.get(&(token.text.to_owned(), value_id_entry.scope)).unwrap().to_owned();
                        
                        if !self.add_code(0xAD) { return false; }
                        if !self.add_var(value_static_offset) { return false; }
                    },
                    TokenType::Digit(val) => {
                        // Digits just load a constant to the accumulator
                        if !self.add_code(0xA9) { return false; }
                        if !self.add_code(*val as u8) { return false; }
                    },
                    TokenType::Char(string) => {
                        // Start by storing the string
                        let addr: Option<u8> = self.store_string(&string);

                        // Store the starting address of the string in memory
                        if addr.is_some() {
                            if !self.add_code (0xA9) { return false; }
                            if !self.add_code(addr.unwrap()) { return false; }
                        } else {
                            return false;
                        }
                    },
                    TokenType::Keyword(keyword) => {
                        match &keyword {
                            Keywords::True => {
                                // True is 0x01
                                if !self.add_code(0xA9) { return false; }
                                if !self.add_code(0x01) { return false; }
                            },
                            Keywords::False => {
                                // False is 0x00
                                if !self.add_code(0xA9) { return false; }
                                if !self.add_code(0x00) { return false; }
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
                        if !self.get_z_flag_value() { return false; }
                    },
                    NonTerminalsAst::NotEq => {
                        if !self.code_gen_compare(ast, children[0], symbol_table, false) { return false; }
                        if !self.get_z_flag_value() { return false; }
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
                let static_offset = self.static_table.get(&(token.text.to_owned(), id_entry.scope)).unwrap().to_owned();
                
                // The data that we are storing is already in the accumulator
                // so just run the code to store the data
                if !self.add_code(0x8D) { return false; }
                if !self.add_var(static_offset) { return false; }
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
                        let static_offset: usize = self.static_table.get(&(id_name.to_owned(), print_id.scope)).unwrap().to_owned();
                        match &print_id.symbol_type {
                            Type::Int  => {
                                // Load the integer value into the Y register
                                if !self.add_code(0xAC) { return false; }
                                if !self.add_var(static_offset) { return false; }

                                // Set X to 1 for the system call
                                if !self.add_code(0xA2) { return false; }
                                if !self.add_code(0x01) { return false; }
                            },
                            Type::String => {
                                // Store the string address in Y
                                if !self.add_code(0xAC) { return false; }
                                if !self.add_var(static_offset) { return false; }

                                // X = 2 for this sys call
                                if !self.add_code(0xA2) { return false; }
                                if !self.add_code(0x02) { return false; }
                            },
                            Type::Boolean => {
                                // Compare the value of the variable with true
                                if !self.add_code(0xA2) { return false; }
                                if !self.add_code(0x01) { return false; }
                                if !self.add_code(0xEC) { return false; }
                                if !self.add_var(static_offset) { return false; }
                                // Skip to the false string if it is false
                                if !self.add_code(0xD0) { return false; }
                                if !self.add_code(0x07) { return false; }
                                
                                // Load the true string and skip over the false string
                                if !self.add_code(0xA0) { return false; }
                                if !self.add_code(*self.string_history.get("true").unwrap()) { return false; }
                                if !self.add_code(0xEC) { return false; }
                                if !self.add_code(0xFF) { return false; }
                                if !self.add_code(0x00) { return false; }
                                if !self.add_code(0xD0) { return false; }
                                if !self.add_code(0x02) { return false; }
                                // Load the false string
                                if !self.add_code(0xA0) { return false; }
                                if !self.add_code(*self.string_history.get("false").unwrap()) { return false; }

                                // We are printing a string, so X = 2
                                if !self.add_code(0xA2) { return false; }
                                if !self.add_code(0x02) { return false; }
                            }
                        }
                    },
                    TokenType::Digit(digit) => {
                        // Sys call 1 for integers needs the number in Y
                        if !self.add_code(0xA0) { return false; }
                        if !self.add_code(*digit as u8) { return false; }

                        // And X = 1
                        if !self.add_code(0xA2) { return false; }
                        if !self.add_code(0x01) { return false; }
                    },
                    TokenType::Char(string) => {
                        // Store the string in memory and load its address to Y
                        let addr: Option<u8> = self.store_string(&string);
                        if addr.is_some() {
                            if !self.add_code(0xA0) { return false; }
                            if !self.add_code(addr.unwrap()) { return false; }
                        } else {
                            return false;
                        }

                        // X = 2 for a string sys call
                        if !self.add_code(0xA2) { return false; }
                        if !self.add_code(0x02) { return false; }
                    },
                    TokenType::Keyword(keyword) => {
                        if !self.add_code(0xA0) { return false; }
                        match keyword {
                            Keywords::True => {
                                // Y = true addr for true
                                if !self.add_code(*self.string_history.get("true").unwrap()) { return false; }
                            },
                            Keywords::False => {
                                // Y = false addr for false
                                if !self.add_code(*self.string_history.get("false").unwrap()) { return false; }
                            },
                            _ => error!("Received {:?} when expecting true or false for print keyword", keyword)
                        }
                        // X = 2 for the sys call
                        if !self.add_code(0xA2) { return false; }
                        if !self.add_code(0x02) { return false; }
                    },
                    _ => error!("Received {:?} when expecting id, digit, string, or keyword for print terminal", token)
                }
            },
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                match non_terminal {
                    NonTerminalsAst::Add => {
                        // Generate the result of the addition expression
                        if !self.code_gen_add(ast, children[0], symbol_table, true) { return false; }

                        let temp_addr_option: Option<usize> = self.new_temp();
                        if temp_addr_option.is_none() {
                            return false;
                        }
                        let temp_addr: usize = temp_addr_option.unwrap();

                        if !self.add_code(0x8D) { return false; }
                        if !self.add_temp(temp_addr) { return false; }
                        
                        // Load the result to Y (wish there was TAY)
                        if !self.add_code(0xAC) { return false; }
                        if !self.add_temp(temp_addr) { return false; }
                        
                        // We are done with the temp data
                        self.temp_index -= 1;

                        // X = 1 for the sys call for integers
                        if !self.add_code(0xA2) { return false; }
                        if !self.add_code(0x01) { return false; }
                    },
                    NonTerminalsAst::IsEq => {
                        // If it is true or false is in the Z flag
                        if !self.code_gen_compare(ast, children[0], symbol_table, true) { return false; }

                        // We are printing a string, so X = 2
                        if !self.add_code(0xA2) { return false; }
                        if !self.add_code(0x02) { return false; }

                        // Skip to the false string if it is false
                        if !self.add_code(0xD0) { return false; }
                        if !self.add_code(0x07) { return false; }
                        
                        // Load the true string and skip over the false string
                        if !self.add_code(0xA0) { return false; }
                        if !self.add_code(*self.string_history.get("true").unwrap()) { return false; }
                        if !self.add_code(0xEC) { return false; }
                        if !self.add_code(0xFF) { return false; }
                        if !self.add_code(0x00) { return false; }
                        if !self.add_code(0xD0) { return false; }
                        if !self.add_code(0x02) { return false; }

                        // Load the false string
                        if !self.add_code(0xA0) { return false; }
                        if !self.add_code(*self.string_history.get("false").unwrap()) { return false; }
                    },
                    NonTerminalsAst::NotEq => {
                        if !self.code_gen_compare(ast, children[0], symbol_table, false) { return false; }
                         // We are printing a string, so X = 2
                        if !self.add_code(0xA2) { return false; }
                        if !self.add_code(0x02) { return false; }

                        // Skip to the false string if it is false
                        if !self.add_code(0xD0) { return false; }
                        if !self.add_code(0x07) { return false; }
                        
                        // Load the true string and skip over the false string
                        if !self.add_code(0xA0) { return false; }
                        if !self.add_code(*self.string_history.get("true").unwrap()) { return false; }
                        if !self.add_code(0xEC) { return false; }
                        if !self.add_code(0xFF) { return false; }
                        if !self.add_code(0x00) { return false; }
                        if !self.add_code(0xD0) { return false; }
                        if !self.add_code(0x02) { return false; }

                        // Load the false string
                        if !self.add_code(0xA0) { return false; }
                        if !self.add_code(*self.string_history.get("false").unwrap()) { return false; }
                   },
                    _ => error!("Received {:?} when expecting addition or boolean expression for nonterminal print", non_terminal)
                }
            },
            _ => error!("Received {:?} when expecting terminal or AST nonterminal for print in code gen", child)
        }

        // The x and y registers are all set up, so just add the sys call
        if !self.add_code(0xFF) { return false; }
        return true;
    }

    // Function to generate code for an addition statement
    // Result is left in the accumulator
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

        // Make some space for the temporary data only if first addition
        // Otherwise, use the current max temp index, which is the working temp location
        let mut temp_addr: usize = self.temp_index - 1;
        if is_first {
            let temp_addr_option: Option<usize> = self.new_temp();
            if temp_addr_option.is_none() {
                return false;
            }
            temp_addr = temp_addr_option.unwrap();
        }

        match right_child {
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Digit(num) => {
                        // Store right side digit in the accumulator
                        if !self.add_code(0xA9) { return false; }
                        if !self.add_code(*num) { return false; }
                    },
                    TokenType::Identifier(_) => {
                        // Get the address needed from memory for the identifier
                        let value_id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 
                        let value_static_offset: usize = self.static_table.get(&(token.text.to_owned(), value_id_entry.scope)).unwrap().to_owned();
                        
                        // Load the value into the accumulator
                        if !self.add_code(0xAD) { return false; }
                        if !self.add_var(value_static_offset) { return false; }
                    },
                    _ => error!("Received {:?} when expecting digit or id for right side of addition", token)
                }

                // Both digits and ids are in the accumulator, so move them to
                // the res address for usage in the math operation
                if !self.add_code(0x8D) { return false; }
                if !self.add_temp(temp_addr) { return false; }
                // We are using a new temporary value for temps, so increment the index
            },
            // Nonterminals are always add, so just call it
            SyntaxTreeNode::NonTerminalAst(_) => if !self.code_gen_add(ast, children[0], symbol_table, false) { return false; },
            _ => error!("Received {:?} when expecting terminal or AST nonterminal for right addition value", right_child)
        }

        match left_child {
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Digit(num) => {
                        // Put left digit in acc
                        if !self.add_code(0xA9) { return false; }
                        if !self.add_code(*num) { return false; }

                        // Perform the addition
                        if !self.add_code(0x6D) { return false; }
                        if !self.add_temp(temp_addr) { return false; }

                        // Only store the result back in memory if we have more addition to do
                        if !is_first {
                            // Store it back in the resulting address
                            if !self.add_code(0x8D) { return false; }
                            if !self.add_temp(temp_addr) { return false; }
                        } else {
                            // We are done with the memory location, so can move
                            // the pointer back over 1
                            self.temp_index -= 1;
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
                    TokenType::Identifier(_) => {
                        // Get the address needed from memory for the identifier
                        let value_id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 
                        let value_static_offset: usize = self.static_table.get(&(token.text.to_owned(), value_id_entry.scope)).unwrap().to_owned();
                        
                        // Load the value into the accumulator
                        if !self.add_code(0xAD) { return false; }
                        if !self.add_var(value_static_offset) { return false; }
                    },
                    TokenType::Digit(num) => {
                        // Store the digit in memory
                        if !self.add_code(0xA9) { return false; }
                        if !self.add_code(*num) { return false; }
                    },
                    TokenType::Char(string) => {
                        let string_addr: Option<u8> = self.store_string(string);
                        if string_addr.is_some() {
                            if !self.add_code(0xA9) { return false; }
                            if !self.add_code(string_addr.unwrap()) { return false; }
                        } else {
                            return false;
                        }
                    },
                    TokenType::Keyword(keyword) => {
                        if !self.add_code(0xA9) { return false; }
                        match &keyword {
                            Keywords::True => if !self.add_code(0x01) { return false; },
                            Keywords::False => if !self.add_code(0x00) { return false; },
                            _ => error!("Received {:?} when expecting true or false for keywords in boolean expression", keyword)
                        }
                    },
                    _ => error!("Received {:?} when expecting an Id, digit, char, or keyword for left side of boolean expression", token)
                }
            },
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                match &non_terminal {
                    NonTerminalsAst::Add => {
                        if !self.code_gen_add(ast, children[1], symbol_table, true) { return false; }
                    },
                    NonTerminalsAst::IsEq => {
                        if !self.code_gen_compare(ast, children[1], symbol_table, true) { return false; }
                        if !self.get_z_flag_value() { return false; }
                    },
                    NonTerminalsAst::NotEq => {
                        if !self.code_gen_compare(ast, children[1], symbol_table, false) { return false; }
                        if !self.get_z_flag_value() { return false; }
                    },
                    _ => error!("Received {:?} for left side of nonterminal boolean expression, when expected Add, IsEq, or NotEq", non_terminal)
                }
            },
            _ => error!("Received {:?} when expected terminal or AST nonterminal for left side of comparison in code gen", left_child)
        }

        // The left hand side is already in the ACC, so can store in temp memory
        let left_temp_option: Option<usize> = self.new_temp();
        if left_temp_option.is_none() {
            return false;
        }
        let left_temp: usize = left_temp_option.unwrap();

        if !self.add_code(0x8D) { return false; }
        if !self.add_temp(left_temp) { return false; }

        match right_child {
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Identifier(_) => {
                        // Get the address needed from memory for the identifier
                        let value_id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 
                        let value_static_offset: usize = self.static_table.get(&(token.text.to_owned(), value_id_entry.scope)).unwrap().to_owned();
                        
                        // Load the value into the X register
                        if !self.add_code(0xAE) { return false; }
                        if !self.add_var(value_static_offset) { return false; }
                    },
                    TokenType::Digit(num) => {
                        // Store the digit in X
                        if !self.add_code(0xA2) { return false; }
                        if !self.add_code(*num) { return false; }
                    },
                    TokenType::Char(string) => {
                        let string_addr: Option<u8> = self.store_string(string);
                        if string_addr.is_some() {
                            if !self.add_code(0xA2) { return false; }
                            if !self.add_code(string_addr.unwrap()) { return false; }
                        } else {
                            return false;
                        }
                    },
                    TokenType::Keyword(keyword) => {
                        if !self.add_code(0xA2) { return false; }
                        match &keyword {
                            Keywords::True => if !self.add_code(0x01) { return false; },
                            Keywords::False => if !self.add_code(0x00) { return false; },
                            _ => error!("Received {:?} when expecting true or false for keywords in boolean expression", keyword)
                        }
                    },
                    _ => error!("Received {:?} when expecting an Id, digit, char, or keyword for left side of boolean expression", token)
                }
            },
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                match &non_terminal {
                    NonTerminalsAst::Add => {
                        if !self.code_gen_add(ast, children[0], symbol_table, true) { return false; }
                    },
                    NonTerminalsAst::IsEq => {
                        if !self.code_gen_compare(ast, children[0], symbol_table, true) { return false; }
                        if !self.get_z_flag_value() { return false; }
                    },
                    NonTerminalsAst::NotEq => {
                        if !self.code_gen_compare(ast, children[0], symbol_table, false) { return false; }
                        if !self.get_z_flag_value() { return false; }
                    },
                    _ => error!("Received {:?} for right side of nonterminal boolean expression, when expected Add, IsEq, or NotEq", non_terminal)
                }

                // The nonterminal result is in the ACC, so have to move to X
                let temp_addr_option: Option<usize> = self.new_temp();
                if temp_addr_option.is_none() {
                    return false;
                }
                let temp_addr: usize = temp_addr_option.unwrap();

                if !self.add_code(0x8D) { return false; }
                if !self.add_temp(temp_addr) { return false; }

                if !self.add_code(0xAE) { return false; }
                if !self.add_temp(temp_addr) { return false; }
                self.temp_index -= 1;
            },
            _ => error!("Received {:?} when expected terminal or AST nonterminal for left side of comparison in code gen", left_child)
        }

        if !self.add_code(0xEC) { return false; }
        if !self.add_temp(left_temp) { return false; }

        // We are done with this data
        self.temp_index -= 1;

        // Add code if the operation is for not equals
        // This effectively flips the Z flag
        if !is_eq {
            // Start assuming that they were not equal
            if !self.add_code(0xA2) { return false; }
            if !self.add_code(0x00) { return false; }
            // Take the branch if not equal
            if !self.add_code(0xD0) { return false; }
            if !self.add_code(0x02) { return false; }
            // If equal, set x to 1
            if !self.add_code(0xA2) { return false; }
            if !self.add_code(0x01) { return false; }
            // Compare with 0 to flip the Z flag
            if !self.add_code(0xEC) { return false; }
            if !self.add_code(0xFF) { return false; }
            if !self.add_code(0x00) { return false; }
        }

        return true;
    }

    // Stores the value of the Z flag into the accumulator
    fn get_z_flag_value(&mut self) -> bool {
        // Assume Z is set to 0
        if !self.add_code(0xA9) { return false; }
        if !self.add_code(0x00) { return false; }
        // If it is 0, branch
        if !self.add_code(0xD0) { return false; }
        if !self.add_code(0x02) { return false; }
        // Otherwise, set the acc to 1
        if !self.add_code(0xA9) { return false; }
        if !self.add_code(0x01) { return false; }

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

        // Starting address for the branch, but 0 will never be valid, so can have
        // default value set to 0
        let mut start_addr: u8 = 0x00;
        // This is the index of the jump that will ultimately be backpatched
        let jump_index: usize = self.jumps.len();

        match left_child {
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                match &non_terminal {
                    // Evaluate the boolean expression for the if statement
                    // The Z flag is set by these function calls
                    NonTerminalsAst::IsEq => if !self.code_gen_compare(ast, children[1], symbol_table, true) { return false; },
                    NonTerminalsAst::NotEq => if !self.code_gen_compare(ast, children[1], symbol_table, false) { return false; },
                    _ => error!("Received {:?} when expecting IsEq or NotEq for nonterminal if expression", non_terminal)
                }
                // Add the branch code
                if !self.add_code(0xD0) { return false; }
                if !self.add_jump() { return false; }
                start_addr = self.code_pointer.to_owned();
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

        // If there was a comparison to make, there is a start addr
        if start_addr != 0x00 {
            // Compute the difference and set it in the vector for use in backpatching
            let branch_offset: u8 = self.code_pointer - start_addr;
            self.jumps[jump_index] = branch_offset;
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

        // Save the current address for the loop
        let loop_start_addr: u8 = self.code_pointer.to_owned();

        // Starting address for the body of the while structure,
        // but 0 will never be valid, so can have default value set to 0
        let mut body_start_addr: u8 = 0x00;
        // This is the index of the body jump if a condition eveluates to false
        // that will ultimately be backpatched
        let body_jump_index: usize = self.jumps.len();

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
                if !self.add_code(0xD0) { return false; }
                if !self.add_jump() { return false; }
                body_start_addr = self.code_pointer.to_owned();
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

        // Get the position in the vector for the unconditional branch
        let unconditional_jump_index: usize = self.jumps.len();
        // Set X to 1
        if !self.add_code(0xA2) { return false; }
        if !self.add_code(0x01) { return false; }
        // 0xFF is always 0, so comparing it to 1 will result in Z = 0,
        // so the branch will always be taken
        if !self.add_code(0xEC) { return false; }
        if !self.add_code(0xFF) { return false; }
        if !self.add_code(0x00) { return false; }
        if !self.add_code(0xD0) { return false; }
        if !self.add_jump() { return false; }

        // If there was a comparison to make, there is a start addr for the body
        // to skip over in case evaluate to false
        if body_start_addr != 0x00 {
            // Compute the difference and set it in the vector for use in backpatching
            let conditional_branch_offset: u8 = self.code_pointer - body_start_addr;
            self.jumps[body_jump_index] = conditional_branch_offset;
        }
        
        // The branch offset is the 2s complement difference between the current position
        // and the start of the loop, so take the difference and negate and add 1
        let unconditional_branch_offset: u8 = !(self.code_pointer - loop_start_addr) + 1;
        // Set the unconditional branch offset in the jump
        self.jumps[unconditional_jump_index] = unconditional_branch_offset;

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

        // Get the array of values but only keep the hex digits and spaces
        let mut code_str: String = format!("{:?}", self.code_arr);
        code_str.retain(|c| c != ',' && c != '[' && c != ']');

        // This is the element that the code is in
        let code_elem: Element = document.create_element("p").expect("Should be able to create the element");
        code_elem.set_class_name("code-text");
        code_elem.set_inner_html(&code_str);

        display_area_div.append_child(&code_elem).expect("Should be able to add the child node");

        // This is the button to copy to the clipboard
        let copy_btn: Element = document.create_element("button").expect("Should be able to create the element");
        copy_btn.set_inner_html("Copy to Clipboard");
        copy_btn.set_class_name("copy-btn");
        display_area_div.append_child(&copy_btn).expect("Should be able to add the child node");

        // Create a function that will be used as the event listener and add it to the copy button
        let copy_btn_fn: Closure<dyn FnMut()> = Closure::wrap(Box::new(move || {
            // Call the JS function that handles the clipboard
            set_clipboard(&code_str);
        }) as Box<dyn FnMut()>);
        copy_btn.add_event_listener_with_callback("click", copy_btn_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
        copy_btn_fn.forget();

        // Add the div to the pane
        content_area.append_child(&display_area_div).expect("Should be able to add the child node");
    }

    pub fn clear_display() {
        // Get the preliminary objects
        let window: Window = web_sys::window().expect("Should be able to get the window");
        let document: Document = window.document().expect("Should be able to get the document");

        // Clear the entire area
        let tabs_area: Element = document.get_element_by_id("code-gen-tabs").expect("Should be able to find the element");
        tabs_area.set_inner_html("");
        let content_area: Element = document.get_element_by_id("code-gen-tab-content").expect("Should be able to find the element");
        content_area.set_inner_html("");
    }
}
