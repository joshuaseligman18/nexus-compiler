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
pub struct CodeGeneratorRiscV {
    // The current max scope we have seen so far, which are encountered in
    // sequential order
    max_scope: usize,
    
    // The array for the code
    code_arr: Vec<String>,

    // The array for the variables
    static_arr: Vec<String>,
    
    // The array for temp data
    temp_arr: Vec<String>,
    
    // The array for strings / heap data
    heap_arr: Vec<String>,

    // The current location of the code in the memory array
    // The stack pointer is always code_pointer + 1
    //code_pointer: u8,

    // The current location of the heap from the back of the array
    //heap_pointer: u8,

    // The static table hashmap for <(id, scope), offset>
    //static_table: HashMap<(String, usize), usize>,

    // Index for the temoprary data
    temp_index: usize,

    // Hashmap to keep track of the strings being stored on the heap
    string_history: HashMap<String, usize>,

    // Vector to keep track of each jump in the code
    //jumps: Vec<u8>,
}

impl CodeGeneratorRiscV {
    pub fn new() -> Self {
        return CodeGeneratorRiscV {
            max_scope: usize::MAX,
            code_arr: Vec::new(),
            static_arr: Vec::new(),
            temp_arr: Vec::new(),
            heap_arr: Vec::new(),
            temp_index: 0,
            string_history: HashMap::new()
        };
    }
//    pub fn new() -> Self {
//        let mut code_gen: CodeGeneratorRiscV = CodeGeneratorRiscV {
//            // This is a flag for a new program
//            max_scope: usize::MAX,
//
//            // We are only able to store 256 bytes in memory
//            code_arr: Vec::with_capacity(0x100),
//
//            // Code starts at 0x00
//            code_pointer: 0x00,
//
//            // Heap starts at 0xFE (0xFF reserved for 0x00)
//            heap_pointer: 0xFE,
//
//            static_table: HashMap::new(),
//
//            // Always start with a temp index of 0
//            temp_index: 0,
//
//            string_history: HashMap::new(),
//
//            jumps: Vec::new()
//        };
//
//        // Initialize the entire array to be unused spot in memory
//        for _ in 0..0x100 {
//            code_gen.code_arr.push(CodeGenBytes::Empty);
//        }
//
//        return code_gen;
//    }

    pub fn generate_code(&mut self, ast: &SyntaxTree, symbol_table: &mut SymbolTable, program_number: &u32) {
        // Make sure the current scope is set to be a flag for none
        self.max_scope = usize::MAX;
        
        self.code_arr.clear();
        self.static_arr.clear();
        self.temp_arr.clear();
        self.heap_arr.clear();

        // Initialize the basic data for printing functionality
        self.heap_arr.push(format!("new_line: .ascii \"\\n\""));
        self.heap_arr.push(format!("print_int_char: .byte 0"));
        
        self.temp_index = 0;
        self.string_history.clear();

        // Store the actual strings "true" and "false"
        self.store_string("false");
        self.store_string("true");

        // We are going to store the strings false and true to print them
        // out instead of 0 and 1
        //self.store_string("false");
        //self.store_string("true");

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
       
        debug!("{:?}", self.code_arr);
        debug!("{:?}", self.static_arr);
        debug!("{:?}", self.temp_arr);
        debug!("{:?}", self.heap_arr);

        info!("{}", self.create_output_string());

//        if program_res {
//            // All programs end with 0x00, which is HALT
//            let final_res: bool = self.add_code(0x00);
//            debug!("{:?}", self.code_arr);
//
//            if final_res {
//                self.backpatch_addresses();
//
//                debug!("Static table: {:?}", self.static_table);
//                debug!("Jumps vector: {:?}", self.jumps);
//                debug!("{:?}", self.code_arr);
//
//                nexus_log::log(
//                    nexus_log::LogTypes::Info,
//                    nexus_log::LogSources::CodeGenerator,
//                    format!("Code generation completed successfully")
//                );
//
//                nexus_log::log(
//                    nexus_log::LogTypes::Info,
//                    nexus_log::LogSources::Nexus,
//                    format!("Executable image for program {} is below", *program_number)
//                );
//
//                self.display_code(program_number);
//                return;
//            }
//        }

//        nexus_log::log(
//            nexus_log::LogTypes::Error,
//            nexus_log::LogSources::CodeGenerator,
//            format!("Code generation failed")
//        );
//        
//        nexus_log::insert_empty_line();
//
//        nexus_log::log(
//            nexus_log::LogTypes::Warning,
//            nexus_log::LogSources::Nexus,
//            format!("Executable image display skipped due to code generation failure")
//        );
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
                        //NonTerminalsAst::If => self.code_gen_if(ast, neighbor_index, symbol_table),
                        //NonTerminalsAst::While => self.code_gen_while(ast, neighbor_index, symbol_table),
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

    fn create_output_string(&mut self) -> String {
        let mut output_builder: Builder = Builder::default();
        
        output_builder.append(".section .text\n");
        output_builder.append(".global _start\n");
        output_builder.append("_start:\n");
        output_builder.append("nop\n");
        for code in self.code_arr.iter() {
            output_builder.append(code.as_str());
            output_builder.append("\n");
        }

        //output_builder.append(".section .data\n");
        for static_data in self.static_arr.iter() {
            output_builder.append(static_data.as_str());
            output_builder.append("\n");
        }

        for heap_data in self.heap_arr.iter() {
            output_builder.append(heap_data.as_str());
            output_builder.append("\n");
        }

        return output_builder.string().unwrap();
    }

//    fn has_available_memory(&mut self) -> bool {
//        let num_vars: usize = self.static_table.len();
//        // Check for collision at the double bar (where stack meets heap)
//        //  |  Code  |  Vars  ||  Temp  |  Heap  |
//        return self.code_pointer + (num_vars as u8) <= self.heap_pointer - (self.temp_index as u8);
//    }
//
//    // Function to add byte of code to the memory array
//    fn add_code(&mut self, code: u8) -> bool {
//        if self.has_available_memory() {
//            nexus_log::log(
//                nexus_log::LogTypes::Debug,
//                nexus_log::LogSources::CodeGenerator,
//                format!("Adding code 0x{:02X} at memory location 0x{:02X}", code, self.code_pointer)
//            );
//
//            // Add the code to the next available spot in memory
//            self.code_arr[self.code_pointer as usize] = CodeGenBytes::Code(code);
//            self.code_pointer += 1;
//            // No error, so successful addition to the code
//            return true;
//        } else {
//            nexus_log::log(
//                nexus_log::LogTypes::Error,
//                nexus_log::LogSources::CodeGenerator,
//                String::from("The stack has collided with the heap causing a stack overflow error")
//            );
//            return false;
//        }
//    }
//
//    // Function to add byte of code to the memory array for variable addressing
//    fn add_var(&mut self, var: usize) -> bool {
//        if self.has_available_memory() {
//            nexus_log::log(
//                nexus_log::LogTypes::Debug,
//                nexus_log::LogSources::CodeGenerator,
//                format!("Adding variable placeholder {} at memory location 0x{:02X}", var, self.code_pointer)
//            );
//
//            // Add the code to the next available spot in memory
//            self.code_arr[self.code_pointer as usize] = CodeGenBytes::Var(var);
//            self.code_pointer += 1;
//            // All vars are followed by the high order byte
//            return self.add_high_order_byte();
//        } else {
//            nexus_log::log(
//                nexus_log::LogTypes::Error,
//                nexus_log::LogSources::CodeGenerator,
//                String::from("The stack has collided with the heap causing a stack overflow error")
//            );
//            return false;
//        }
//    }
//
//    // Function to add the high order byte for unknown addresses that will be backpatched
//    fn add_high_order_byte(&mut self) -> bool {
//        if self.has_available_memory() {
//            nexus_log::log(
//                nexus_log::LogTypes::Debug,
//                nexus_log::LogSources::CodeGenerator,
//                format!("Adding high order byte placeholder at memory location 0x{:02X}", self.code_pointer)
//            );
//
//            // Add the code to the next available spot in memory
//            self.code_arr[self.code_pointer as usize] = CodeGenBytes::HighOrderByte;
//            self.code_pointer += 1;
//            return true;
//        } else {
//            nexus_log::log(
//                nexus_log::LogTypes::Error,
//                nexus_log::LogSources::CodeGenerator,
//                String::from("The stack has collided with the heap causing a stack overflow error")
//            );
//            return false;
//        }
//    }
//
//    // Function to create space for new temp data and return its index
//    fn new_temp(&mut self) -> Option<usize> {
//        if self.has_available_memory() {
//            // Make the room for the single byte
//            let temp_addr: usize = self.temp_index.to_owned();
//            self.temp_index += 1;
//            return Some(temp_addr);
//        } else {
//            nexus_log::log(
//                nexus_log::LogTypes::Error,
//                nexus_log::LogSources::CodeGenerator,
//                String::from("The heap has collided with the stack causing a heap overflow error")
//            );
//            return None;
//        }
//    }
//
//    // Function to add byte of code to memory array for temporary data
//    fn add_temp(&mut self, temp: usize) -> bool {
//        if self.has_available_memory() {
//            nexus_log::log(
//                nexus_log::LogTypes::Debug,
//                nexus_log::LogSources::CodeGenerator,
//                format!("Adding temp data placeholder {} at memory location 0x{:02X}", temp, self.code_pointer)
//            );
//
//            // Add the addressing for the temporary value
//            self.code_arr[self.code_pointer as usize] = CodeGenBytes::Temp(temp);
//            self.code_pointer += 1;
//            // All temps are followed by the high order byte
//            return self.add_high_order_byte();
//        } else {
//            nexus_log::log(
//                nexus_log::LogTypes::Error,
//                nexus_log::LogSources::CodeGenerator,
//                String::from("The heap has collided with the stack causing a heap overflow error")
//            );
//            return false;
//        }
//    }
//
//    // Function to add a byte of data to the heap
//    fn add_data(&mut self, data: u8) -> bool {
//        if self.has_available_memory() {
//            nexus_log::log(
//                nexus_log::LogTypes::Debug,
//                nexus_log::LogSources::CodeGenerator,
//                format!("Adding data 0x{:02X} at memory location 0x{:02X}", data, self.heap_pointer)
//            );
//
//            // Heap starts from the end of the 256 bytes and moves towards the front
//            self.code_arr[self.heap_pointer as usize] = CodeGenBytes::Data(data);
//            self.heap_pointer -= 1;
//            return true;
//        } else {
//            nexus_log::log(
//                nexus_log::LogTypes::Error,
//                nexus_log::LogSources::CodeGenerator,
//                String::from("The heap has collided with the stack causing a heap overflow error")
//            );
//            return false;
//        }
//    }

    fn store_string(&mut self, string: &str) -> usize {
        let addr: Option<&usize> = self.string_history.get(string);
        if addr.is_none() {
            // Place the string in the heap
            self.heap_arr.push(format!("string_{}:", self.string_history.len()));
            // We will let strings be no longer than 2^16 - 1
            self.heap_arr.push(format!(".half {}", string.len()));
            self.heap_arr.push(format!(".ascii \"{}\"", string));
//            self.heap_arr.push(format!("string_{}: .2byte {} .ascii \"{}\"", self.string_history.len(), string.len(), string));
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

//    fn add_jump(&mut self) -> bool {
//        if self.has_available_memory() {
//            nexus_log::log(
//                nexus_log::LogTypes::Debug,
//                nexus_log::LogSources::CodeGenerator,
//                format!("Adding jump placeholder {} at memory location 0x{:02X}", self.jumps.len(), self.code_pointer)
//            );
//
//            // Add the jump to the code and set it to 0 in the vector of jumps
//            self.code_arr[self.code_pointer as usize] = CodeGenBytes::Jump(self.jumps.len());
//            self.code_pointer += 1;
//            self.jumps.push(0x00);
//            return true;
//        } else {
//            nexus_log::log(
//                nexus_log::LogTypes::Error,
//                nexus_log::LogSources::CodeGenerator,
//                String::from("The stack has collided with the heap causing a stack overflow error")
//            );
//            return false;
//        }
//    }
//
//    // Replaces temp addresses with the actual position in memory
//    // Do not have to worry about memory availability because that was taken
//    // care of when the placeholders were created
//    fn backpatch_addresses(&mut self) { 
//        for i in 0..self.code_arr.len() {
//            match &self.code_arr[i] {
//                CodeGenBytes::Var(offset) => {
//                    // Compute the new address
//                    let new_addr: u8 = self.code_pointer + *offset as u8;
//                    nexus_log::log(
//                        nexus_log::LogTypes::Debug,
//                        nexus_log::LogSources::CodeGenerator,
//                        format!("Backpatching 0x{:02X} for variable placeholder {} at memory location 0x{:02X}", new_addr, offset, i)
//                    );
//
//                    self.code_arr[i] = CodeGenBytes::Code(new_addr);
//
//                    // The integer division result is the high order byte
//                    // Always 0 in this case
//                    let new_high: u8 = (new_addr as u16 / 0x100) as u8;
//
//                    nexus_log::log(
//                        nexus_log::LogTypes::Debug,
//                        nexus_log::LogSources::CodeGenerator,
//                        format!("Backpatching 0x{:02X} for high order byte placeholder at memory location 0x{:02X}", new_high, i + 1)
//                    );
//
//                    self.code_arr[i + 1] = CodeGenBytes::Code(new_high);
//                },
//                CodeGenBytes::Temp(offset) => {
//                    // Compute the address of the temp data
//                    let new_addr: u8 = self.heap_pointer - *offset as u8;
//                    
//                    nexus_log::log(
//                        nexus_log::LogTypes::Debug,
//                        nexus_log::LogSources::CodeGenerator,
//                        format!("Backpatching 0x{:02X} for temp data placeholder {} at memory location 0x{:02X}", new_addr, offset, i)
//                    );
//
//                    self.code_arr[i] = CodeGenBytes::Code(new_addr);
//                   
//                    // The integer division result is the high order byte
//                    // Always 0 in this case
//                    let new_high: u8 = (new_addr as u16 / 0x100) as u8;
//
//                    nexus_log::log(
//                        nexus_log::LogTypes::Debug,
//                        nexus_log::LogSources::CodeGenerator,
//                        format!("Backpatching 0x{:02X} for high order byte placeholder at memory location 0x{:02X}", new_high, i + 1)
//                    );
//
//                    self.code_arr[i + 1] = CodeGenBytes::Code(new_high);
//                },
//                // Store the value from the jump into the placeholder
//                CodeGenBytes::Jump(jump_index) => {
//                    nexus_log::log(
//                        nexus_log::LogTypes::Debug,
//                        nexus_log::LogSources::CodeGenerator,
//                        format!("Backpatching 0x{:02X} for jump placeholder {} at memory location 0x{:02X}", 
//                                self.jumps[*jump_index], *jump_index, i)
//                    );
//                    self.code_arr[i] = CodeGenBytes::Code(self.jumps[*jump_index])
//                },
//                _ => {} 
//            }
//        }
//    }

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
//                    NonTerminalsAst::IsEq => {
//                        if !self.code_gen_compare(ast, children[0], symbol_table, true) { return false; }
//                        if !self.get_z_flag_value() { return false; }
//                    },
//                    NonTerminalsAst::NotEq => {
//                        if !self.code_gen_compare(ast, children[0], symbol_table, false) { return false; }
//                        if !self.get_z_flag_value() { return false; }
//                    },
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
//                    NonTerminalsAst::IsEq => {
//                        // If it is true or false is in the Z flag
//                        if !self.code_gen_compare(ast, children[0], symbol_table, true) { return false; }
//
//                        // We are printing a string, so X = 2
//                        if !self.add_code(0xA2) { return false; }
//                        if !self.add_code(0x02) { return false; }
//
//                        // Skip to the false string if it is false
//                        if !self.add_code(0xD0) { return false; }
//                        if !self.add_code(0x07) { return false; }
//                        
//                        // Load the true string and skip over the false string
//                        if !self.add_code(0xA0) { return false; }
//                        if !self.add_code(*self.string_history.get("true").unwrap()) { return false; }
//                        if !self.add_code(0xEC) { return false; }
//                        if !self.add_code(0xFF) { return false; }
//                        if !self.add_code(0x00) { return false; }
//                        if !self.add_code(0xD0) { return false; }
//                        if !self.add_code(0x02) { return false; }
//
//                        // Load the false string
//                        if !self.add_code(0xA0) { return false; }
//                        if !self.add_code(*self.string_history.get("false").unwrap()) { return false; }
//                    },
//                    NonTerminalsAst::NotEq => {
//                        if !self.code_gen_compare(ast, children[0], symbol_table, false) { return false; }
//                         // We are printing a string, so X = 2
//                        if !self.add_code(0xA2) { return false; }
//                        if !self.add_code(0x02) { return false; }
//
//                        // Skip to the false string if it is false
//                        if !self.add_code(0xD0) { return false; }
//                        if !self.add_code(0x07) { return false; }
//                        
//                        // Load the true string and skip over the false string
//                        if !self.add_code(0xA0) { return false; }
//                        if !self.add_code(*self.string_history.get("true").unwrap()) { return false; }
//                        if !self.add_code(0xEC) { return false; }
//                        if !self.add_code(0xFF) { return false; }
//                        if !self.add_code(0x00) { return false; }
//                        if !self.add_code(0xD0) { return false; }
//                        if !self.add_code(0x02) { return false; }
//
//                        // Load the false string
//                        if !self.add_code(0xA0) { return false; }
//                        if !self.add_code(*self.string_history.get("false").unwrap()) { return false; }
//                   },
                    _ => error!("Received {:?} when expecting addition or boolean expression for nonterminal print", non_terminal)
                }
            },
            _ => error!("Received {:?} when expecting terminal or AST nonterminal for print in code gen", child)
        }

        // Add a new line for cleanliness
        self.code_arr.push(format!("li  a7, 64"));
        self.code_arr.push(format!("li  a0, 1"));
        self.code_arr.push(format!("la  a1, new_line"));
        self.code_arr.push(format!("li  a2, 1"));
        self.code_arr.push(format!("ecall"));

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
                        self.code_arr.push(format!("li  t0, {}", num));
                        if is_first {
                            self.code_arr.push(format!("add  t0, t0, t1"));
                        } else {
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

//    // Function to generate code for comparisons
//    // Result is left in the Z flag and get_z_flag_vale function can be used
//    // afterwards to place z flag value into the accumulator
//    fn code_gen_compare(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable, is_eq: bool) -> bool {
//        nexus_log::log(
//            nexus_log::LogTypes::Debug,
//            nexus_log::LogSources::CodeGenerator,
//            format!("Starting code generation for comparison expression (is_eq = {}) in scope {}", is_eq, symbol_table.cur_scope.unwrap())
//        );
//
//        // Get the child for comparison
//        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
//        let right_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[0]).unwrap();
//        let left_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[1]).unwrap();
//
//        match left_child {
//            SyntaxTreeNode::Terminal(token) => {
//                match &token.token_type {
//                    TokenType::Identifier(_) => {
//                        // Get the address needed from memory for the identifier
//                        let value_id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 
//                        let value_static_offset: usize = self.static_table.get(&(token.text.to_owned(), value_id_entry.scope)).unwrap().to_owned();
//                        
//                        // Load the value into the accumulator
//                        if !self.add_code(0xAD) { return false; }
//                        if !self.add_var(value_static_offset) { return false; }
//                    },
//                    TokenType::Digit(num) => {
//                        // Store the digit in memory
//                        if !self.add_code(0xA9) { return false; }
//                        if !self.add_code(*num) { return false; }
//                    },
//                    TokenType::Char(string) => {
//                        let string_addr: Option<u8> = self.store_string(string);
//                        if string_addr.is_some() {
//                            if !self.add_code(0xA9) { return false; }
//                            if !self.add_code(string_addr.unwrap()) { return false; }
//                        } else {
//                            return false;
//                        }
//                    },
//                    TokenType::Keyword(keyword) => {
//                        if !self.add_code(0xA9) { return false; }
//                        match &keyword {
//                            Keywords::True => if !self.add_code(0x01) { return false; },
//                            Keywords::False => if !self.add_code(0x00) { return false; },
//                            _ => error!("Received {:?} when expecting true or false for keywords in boolean expression", keyword)
//                        }
//                    },
//                    _ => error!("Received {:?} when expecting an Id, digit, char, or keyword for left side of boolean expression", token)
//                }
//            },
//            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
//                match &non_terminal {
//                    NonTerminalsAst::Add => {
//                        if !self.code_gen_add(ast, children[1], symbol_table, true) { return false; }
//                    },
//                    NonTerminalsAst::IsEq => {
//                        if !self.code_gen_compare(ast, children[1], symbol_table, true) { return false; }
//                        if !self.get_z_flag_value() { return false; }
//                    },
//                    NonTerminalsAst::NotEq => {
//                        if !self.code_gen_compare(ast, children[1], symbol_table, false) { return false; }
//                        if !self.get_z_flag_value() { return false; }
//                    },
//                    _ => error!("Received {:?} for left side of nonterminal boolean expression, when expected Add, IsEq, or NotEq", non_terminal)
//                }
//            },
//            _ => error!("Received {:?} when expected terminal or AST nonterminal for left side of comparison in code gen", left_child)
//        }
//
//        // The left hand side is already in the ACC, so can store in temp memory
//        let left_temp_option: Option<usize> = self.new_temp();
//        if left_temp_option.is_none() {
//            return false;
//        }
//        let left_temp: usize = left_temp_option.unwrap();
//
//        if !self.add_code(0x8D) { return false; }
//        if !self.add_temp(left_temp) { return false; }
//
//        match right_child {
//            SyntaxTreeNode::Terminal(token) => {
//                match &token.token_type {
//                    TokenType::Identifier(_) => {
//                        // Get the address needed from memory for the identifier
//                        let value_id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 
//                        let value_static_offset: usize = self.static_table.get(&(token.text.to_owned(), value_id_entry.scope)).unwrap().to_owned();
//                        
//                        // Load the value into the X register
//                        if !self.add_code(0xAE) { return false; }
//                        if !self.add_var(value_static_offset) { return false; }
//                    },
//                    TokenType::Digit(num) => {
//                        // Store the digit in X
//                        if !self.add_code(0xA2) { return false; }
//                        if !self.add_code(*num) { return false; }
//                    },
//                    TokenType::Char(string) => {
//                        let string_addr: Option<u8> = self.store_string(string);
//                        if string_addr.is_some() {
//                            if !self.add_code(0xA2) { return false; }
//                            if !self.add_code(string_addr.unwrap()) { return false; }
//                        } else {
//                            return false;
//                        }
//                    },
//                    TokenType::Keyword(keyword) => {
//                        if !self.add_code(0xA2) { return false; }
//                        match &keyword {
//                            Keywords::True => if !self.add_code(0x01) { return false; },
//                            Keywords::False => if !self.add_code(0x00) { return false; },
//                            _ => error!("Received {:?} when expecting true or false for keywords in boolean expression", keyword)
//                        }
//                    },
//                    _ => error!("Received {:?} when expecting an Id, digit, char, or keyword for left side of boolean expression", token)
//                }
//            },
//            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
//                match &non_terminal {
//                    NonTerminalsAst::Add => {
//                        if !self.code_gen_add(ast, children[0], symbol_table, true) { return false; }
//                    },
//                    NonTerminalsAst::IsEq => {
//                        if !self.code_gen_compare(ast, children[0], symbol_table, true) { return false; }
//                        if !self.get_z_flag_value() { return false; }
//                    },
//                    NonTerminalsAst::NotEq => {
//                        if !self.code_gen_compare(ast, children[0], symbol_table, false) { return false; }
//                        if !self.get_z_flag_value() { return false; }
//                    },
//                    _ => error!("Received {:?} for right side of nonterminal boolean expression, when expected Add, IsEq, or NotEq", non_terminal)
//                }
//
//                // The nonterminal result is in the ACC, so have to move to X
//                let temp_addr_option: Option<usize> = self.new_temp();
//                if temp_addr_option.is_none() {
//                    return false;
//                }
//                let temp_addr: usize = temp_addr_option.unwrap();
//
//                if !self.add_code(0x8D) { return false; }
//                if !self.add_temp(temp_addr) { return false; }
//
//                if !self.add_code(0xAE) { return false; }
//                if !self.add_temp(temp_addr) { return false; }
//                self.temp_index -= 1;
//            },
//            _ => error!("Received {:?} when expected terminal or AST nonterminal for left side of comparison in code gen", left_child)
//        }
//
//        if !self.add_code(0xEC) { return false; }
//        if !self.add_temp(left_temp) { return false; }
//
//        // We are done with this data
//        self.temp_index -= 1;
//
//        // Add code if the operation is for not equals
//        // This effectively flips the Z flag
//        if !is_eq {
//            // Start assuming that they were not equal
//            if !self.add_code(0xA2) { return false; }
//            if !self.add_code(0x00) { return false; }
//            // Take the branch if not equal
//            if !self.add_code(0xD0) { return false; }
//            if !self.add_code(0x02) { return false; }
//            // If equal, set x to 1
//            if !self.add_code(0xA2) { return false; }
//            if !self.add_code(0x01) { return false; }
//            // Compare with 0 to flip the Z flag
//            if !self.add_code(0xEC) { return false; }
//            if !self.add_code(0xFF) { return false; }
//            if !self.add_code(0x00) { return false; }
//        }
//
//        return true;
//    }
//
//    // Stores the value of the Z flag into the accumulator
//    fn get_z_flag_value(&mut self) -> bool {
//        // Assume Z is set to 0
//        if !self.add_code(0xA9) { return false; }
//        if !self.add_code(0x00) { return false; }
//        // If it is 0, branch
//        if !self.add_code(0xD0) { return false; }
//        if !self.add_code(0x02) { return false; }
//        // Otherwise, set the acc to 1
//        if !self.add_code(0xA9) { return false; }
//        if !self.add_code(0x01) { return false; }
//
//        return true;
//    }
//
//    fn code_gen_if(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) -> bool {
//        nexus_log::log(
//            nexus_log::LogTypes::Debug,
//            nexus_log::LogSources::CodeGenerator,
//            format!("Starting code generation for if statement in scope {}", symbol_table.cur_scope.unwrap())
//        );
//
//        // Get the child for comparison
//        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
//        let left_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[1]).unwrap();
//
//        // Starting address for the branch, but 0 will never be valid, so can have
//        // default value set to 0
//        let mut start_addr: u8 = 0x00;
//        // This is the index of the jump that will ultimately be backpatched
//        let jump_index: usize = self.jumps.len();
//
//        match left_child {
//            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
//                match &non_terminal {
//                    // Evaluate the boolean expression for the if statement
//                    // The Z flag is set by these function calls
//                    NonTerminalsAst::IsEq => if !self.code_gen_compare(ast, children[1], symbol_table, true) { return false; },
//                    NonTerminalsAst::NotEq => if !self.code_gen_compare(ast, children[1], symbol_table, false) { return false; },
//                    _ => error!("Received {:?} when expecting IsEq or NotEq for nonterminal if expression", non_terminal)
//                }
//                // Add the branch code
//                if !self.add_code(0xD0) { return false; }
//                if !self.add_jump() { return false; }
//                start_addr = self.code_pointer.to_owned();
//            },
//            SyntaxTreeNode::Terminal(token) => {
//                match &token.token_type {
//                    TokenType::Keyword(Keywords::True) => { /* Small optimization because no comparison is needed */ }
//                    TokenType::Keyword(Keywords::False) => {
//                        // No code should be generated here because the if-statement is just dead
//                        // code and will never be reached, so no point in trying to store the code
//                        // with the limited space that we already have (256 bytes)
//                        return true;
//                    }
//                    _ => error!("Received {:?} when expecting true or false for if expression terminals", token)
//                }
//            },
//            _ => error!("Received {:?} when expecting AST nonterminal or a terminal", left_child)
//        }
//
//        // Generate the code for the body
//        if !self.code_gen_block(ast, children[0], symbol_table) { return false; }
//
//        // If there was a comparison to make, there is a start addr
//        if start_addr != 0x00 {
//            // Compute the difference and set it in the vector for use in backpatching
//            let branch_offset: u8 = self.code_pointer - start_addr;
//            self.jumps[jump_index] = branch_offset;
//        }
//
//        return true;
//    }
//
//    fn code_gen_while(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) -> bool {
//         nexus_log::log(
//            nexus_log::LogTypes::Debug,
//            nexus_log::LogSources::CodeGenerator,
//            format!("Starting code generation for while statement in scope {}", symbol_table.cur_scope.unwrap())
//        );
//
//        // Get the child for comparison
//        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
//        let left_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[1]).unwrap();
//
//        // Save the current address for the loop
//        let loop_start_addr: u8 = self.code_pointer.to_owned();
//
//        // Starting address for the body of the while structure,
//        // but 0 will never be valid, so can have default value set to 0
//        let mut body_start_addr: u8 = 0x00;
//        // This is the index of the body jump if a condition eveluates to false
//        // that will ultimately be backpatched
//        let body_jump_index: usize = self.jumps.len();
//
//        match left_child {
//            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
//                match &non_terminal {
//                    // Evaluate the boolean expression for the while statement
//                    // The Z flag is set by these function calls
//                    NonTerminalsAst::IsEq => if !self.code_gen_compare(ast, children[1], symbol_table, true) { return false; },
//                    NonTerminalsAst::NotEq => if !self.code_gen_compare(ast, children[1], symbol_table, false) { return false; },
//                    _ => error!("Received {:?} when expecting IsEq or NotEq for nonterminal if expression", non_terminal)
//                }
//                // Add the branch code
//                if !self.add_code(0xD0) { return false; }
//                if !self.add_jump() { return false; }
//                body_start_addr = self.code_pointer.to_owned();
//            },
//            SyntaxTreeNode::Terminal(token) => {
//                match &token.token_type {
//                    TokenType::Keyword(Keywords::True) => { /* Small optimization because no comparison is needed */ }
//                    TokenType::Keyword(Keywords::False) => {
//                        // No code should be generated here because the while-statement is just dead
//                        // code and will never be reached, so no point in trying to store the code
//                        // with the limited space that we already have (256 bytes)
//                        return true;
//                    }
//                    _ => error!("Received {:?} when expecting true or false for while expression terminals", token)
//                }
//            },
//            _ => error!("Received {:?} when expecting AST nonterminal or a terminal", left_child)
//        }
//
//        // Generate the code for the body
//        if !self.code_gen_block(ast, children[0], symbol_table) { return false; }
//
//        // Get the position in the vector for the unconditional branch
//        let unconditional_jump_index: usize = self.jumps.len();
//        // Set X to 1
//        if !self.add_code(0xA2) { return false; }
//        if !self.add_code(0x01) { return false; }
//        // 0xFF is always 0, so comparing it to 1 will result in Z = 0,
//        // so the branch will always be taken
//        if !self.add_code(0xEC) { return false; }
//        if !self.add_code(0xFF) { return false; }
//        if !self.add_code(0x00) { return false; }
//        if !self.add_code(0xD0) { return false; }
//        if !self.add_jump() { return false; }
//
//        // If there was a comparison to make, there is a start addr for the body
//        // to skip over in case evaluate to false
//        if body_start_addr != 0x00 {
//            // Compute the difference and set it in the vector for use in backpatching
//            let conditional_branch_offset: u8 = self.code_pointer - body_start_addr;
//            self.jumps[body_jump_index] = conditional_branch_offset;
//        }
//        
//        // The branch offset is the 2s complement difference between the current position
//        // and the start of the loop, so take the difference and negate and add 1
//        let unconditional_branch_offset: u8 = !(self.code_pointer - loop_start_addr) + 1;
//        // Set the unconditional branch offset in the jump
//        self.jumps[unconditional_jump_index] = unconditional_branch_offset;
//
//        return true;
//    }
//
//    fn display_code(&mut self, program_number: &u32) {
//        let window: Window = web_sys::window().expect("Should be able to get the window");
//        let document: Document = window.document().expect("Should be able to get the document");
//
//        let code_gen_tabs: Element = document.get_element_by_id("code-gen-tabs").expect("Should be able to get the element");
//
//        // Create the new tab in the list
//        let new_li: Element = document.create_element("li").expect("Should be able to create the li element");
//
//        // Add the appropriate classes
//        let li_classes: DomTokenList = new_li.class_list();
//        li_classes.add_1("nav-item").expect("Should be able to add the class");
//        new_li.set_attribute("role", "presentation").expect("Should be able to add the attribute");
//
//        // Create the button
//        let new_button: Element = document.create_element("button").expect("Should be able to create the button");
//        let btn_classes: DomTokenList = new_button.class_list();
//        btn_classes.add_1("nav-link").expect("Should be able to add the class");
//
//        // Only make the first one active
//        if code_gen_tabs.child_element_count() == 0 {
//            btn_classes.add_1("active").expect("Should be able to add the class");
//            new_button.set_attribute("aria-selected", "true").expect("Should be able to add the attribute");
//        } else {
//            new_button.set_attribute("aria-selected", "false").expect("Should be able to add the attribute");
//        }
//
//        // Set the id of the button
//        new_button.set_id(format!("program{}-code-gen-btn", *program_number).as_str());
//
//        // All of the toggle elements from the example above
//        new_button.set_attribute("data-bs-toggle", "tab").expect("Should be able to add the attribute");
//        new_button.set_attribute("type", "button").expect("Should be able to add the attribute");
//        new_button.set_attribute("role", "tab").expect("Should be able to add the attribute");
//        new_button.set_attribute("data-bs-target", format!("#program{}-code-gen-pane", *program_number).as_str()).expect("Should be able to add the attribute");
//        new_button.set_attribute("aria-controls", format!("program{}-code-gen-pane", *program_number).as_str()).expect("Should be able to add the attribute");
//
//        // Set the inner text
//        new_button.set_inner_html(format!("Program {}", *program_number).as_str());
//
//        // Append the button and the list element to the area
//        new_li.append_child(&new_button).expect("Should be able to add the child node");
//        code_gen_tabs.append_child(&new_li).expect("Should be able to add the child node");
//
//        // Get the content area
//        let content_area: Element = document.get_element_by_id("code-gen-tab-content").expect("Should be able to find the element");
//
//        // Create the individual pane div
//        let display_area_div: Element = document.create_element("div").expect("Should be able to create the element");
//
//        // Also from the example link above to only let the first pane initially show and be active
//        let display_area_class_list: DomTokenList = display_area_div.class_list();
//        display_area_class_list.add_1("tab-pane").expect("Should be able to add the class");
//        if content_area.child_element_count() == 0 {
//            display_area_class_list.add_2("show", "active").expect("Should be able to add the classes");
//        }
//
//        // Add the appropriate attributes
//        display_area_div.set_attribute("role", "tabpanel").expect("Should be able to add the attribute");
//        display_area_div.set_attribute("tabindex", "0").expect("Should be able to add the attribute");
//        display_area_div.set_attribute("aria-labeledby", format!("program{}-code-gen-btn", *program_number).as_str()).expect("Should be able to add the attribute");
//
//        // Set the id of the pane
//        display_area_div.set_id(format!("program{}-code-gen-pane", *program_number).as_str());
//
//        // The div is a container for the content of the ast info
//        display_area_class_list.add_3("container", "text-center", "code-gen-pane").expect("Should be able to add the classes");
//
//        // Get the array of values but only keep the hex digits and spaces
//        let mut code_str: String = format!("{:?}", self.code_arr);
//        code_str.retain(|c| c != ',' && c != '[' && c != ']');
//
//        // This is the element that the code is in
//        let code_elem: Element = document.create_element("p").expect("Should be able to create the element");
//        code_elem.set_class_name("code-text");
//        code_elem.set_inner_html(&code_str);
//
//        display_area_div.append_child(&code_elem).expect("Should be able to add the child node");
//
//        // This is the button to copy to the clipboard
//        let copy_btn: Element = document.create_element("button").expect("Should be able to create the element");
//        copy_btn.set_inner_html("Copy to Clipboard");
//        copy_btn.set_class_name("copy-btn");
//        display_area_div.append_child(&copy_btn).expect("Should be able to add the child node");
//
//        // Create a function that will be used as the event listener and add it to the copy button
//        let copy_btn_fn: Closure<dyn FnMut()> = Closure::wrap(Box::new(move || {
//            // Call the JS function that handles the clipboard
//            set_clipboard(&code_str);
//        }) as Box<dyn FnMut()>);
//        copy_btn.add_event_listener_with_callback("click", copy_btn_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
//        copy_btn_fn.forget();
//
//        // Add the div to the pane
//        content_area.append_child(&display_area_div).expect("Should be able to add the child node");
//    }
//
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
