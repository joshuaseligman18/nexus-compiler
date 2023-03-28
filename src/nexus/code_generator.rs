use log::*;

use crate::nexus::{syntax_tree::SyntaxTree, syntax_tree_node::*, symbol_table::*};
use crate::nexus::token::{TokenType, Keywords};
use petgraph::graph::{NodeIndex};

use std::collections::HashMap;
use std::fmt;

enum CodeGenBytes {
    // Representation for final code/data in memory
    Code(u8),
    // Temporary code until AST is traversed with identifier for later use
    Temp(usize),
    // Spot is available for anything to take it
    Empty
}

// Customize the output when printing the string
impl fmt::Debug for CodeGenBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CodeGenBytes::Code(code) => write!(f, "{:02X}", code),
            CodeGenBytes::Temp(temp) => write!(f, "T{}", temp),
            CodeGenBytes::Empty => write!(f, "00")
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
    static_table: HashMap<(String, usize), usize>
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

            // Heap starts at 0xFF
            heap_pointer: 0xFF,

            static_table: HashMap::new()
        };

        // Initialize the entire array to be unused spot in memory
        for i in 0..0x100 {
            code_gen.code_arr.push(CodeGenBytes::Empty);
        }

        return code_gen;
    }

    pub fn generate_code(&mut self, ast: &SyntaxTree, symbol_table: &mut SymbolTable) {
        debug!("Code gen called");

        // Make sure the current scope is set to be a flag for none
        self.max_scope = usize::MAX;
        
        // Reset the array and empty it out
        for i in 0..0x100 {
            self.code_arr[i] = CodeGenBytes::Empty;
        }

        self.code_pointer = 0x00;
        self.heap_pointer = 0xFF;

        self.static_table.clear();

        // Generate the code for the program
        self.code_gen_block(ast, NodeIndex::new((*ast).root.unwrap()), symbol_table);
        // All programs end with 0x00, which is HALT
        self.add_code(0x00);

        debug!("{:?}", self.static_table); 
        debug!("{:?}", self.code_arr);
    }

    fn code_gen_block(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) {
        // If this is the first block, then the first scope is 0
        if self.max_scope == usize::MAX {
            self.max_scope = 0;
        } else {
            // Otherwise just add 1
            self.max_scope += 1;
        }
        // Manually set the current scope because we are not able to look down
        // in the symbol table
        symbol_table.set_cur_scope(self.max_scope);

        // The current node is the block, so we need to loop through each of its children
        let neighbors: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();

        for neighbor_index in neighbors.into_iter().rev() {
            debug!("{:?}", (*ast).graph.node_weight(neighbor_index).unwrap());
            let child: &SyntaxTreeNode = (*ast).graph.node_weight(neighbor_index).unwrap();
            
            match child {
                SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                    match non_terminal {
                        NonTerminalsAst::Block => self.code_gen_block(ast, neighbor_index, symbol_table),
                        NonTerminalsAst::VarDecl => self.code_gen_var_decl(ast, neighbor_index, symbol_table),
                        NonTerminalsAst::Assign => self.code_gen_assignment(ast, neighbor_index, symbol_table),
                        NonTerminalsAst::Print => self.code_gen_print(ast, neighbor_index, symbol_table),
                        _ => error!("Received {:?} when expecting an AST nonterminal statement in a block", non_terminal)
                    }
                }
                _ => error!("Received {:?} when expecting an AST nonterminal for code gen in a block", child)
            }
        }

        // Exit the current scope
        symbol_table.end_cur_scope();
    }

    // Function to add byte of code to the memory array
    fn add_code(&mut self, code: u8) {
        // Add the code to the next available spot in memory
        self.code_arr[self.code_pointer as usize] = CodeGenBytes::Code(code);
        self.code_pointer += 1;
    }

    // Function to add byte of code to the memory array
    fn add_temp(&mut self, temp: usize) {
        // Add the code to the next available spot in memory
        self.code_arr[self.code_pointer as usize] = CodeGenBytes::Temp(temp);
        self.code_pointer += 1;
    }


    // Function for creating the code for a variable declaration
    fn code_gen_var_decl(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) {
        debug!("Code gen var decl");
        
        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
        let id_node: &SyntaxTreeNode = (*ast).graph.node_weight(children[0]).unwrap();

        match id_node {
            SyntaxTreeNode::Terminal(token) => {
                debug!("{:?}; {:?}", token.text, symbol_table.cur_scope.unwrap());
                // Get the offset this variable will be on the stack
                let static_offset: usize = self.static_table.len();
                self.static_table.insert((token.text.to_owned(), symbol_table.cur_scope.unwrap()), static_offset);

                // Generate the code for the variable declaration
                self.add_code(0xA9);
                self.add_code(0x00);
                self.add_code(0x8D);
                self.add_temp(static_offset);
                self.add_code(0x00);
            },
            _ => error!("Received {:?} when expecting terminal for var decl child in code gen", id_node)
        }
    }

    // Function for creating the code for an assignment
    fn code_gen_assignment(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) {
        debug!("Code gen assignment");

        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
        let value_node: &SyntaxTreeNode = (*ast).graph.node_weight(children[0]).unwrap();
        let id_node: &SyntaxTreeNode = (*ast).graph.node_weight(children[1]).unwrap();

        match value_node {
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Identifier(id_name) => {
                        debug!("Assignment id");
                    },
                    TokenType::Digit(val) => {
                        debug!("Assignment digit");
                        // Digits just load a constant to the accumulator
                        self.add_code(0xA9);
                        self.add_code(*val as u8);
                    },
                    TokenType::Char(_) => {
                        debug!("Assignment string");
                    },
                    TokenType::Keyword(keyword) => {
                        match &keyword {
                            Keywords::True => {
                                debug!("Assignment true");
                                // True is 0x01
                                self.add_code(0xA9);
                                self.add_code(0x01);
                            },
                            Keywords::False => {
                                debug!("Assignment false");
                                // False is 0x00
                                self.add_code(0xA9);
                                self.add_code(0x00);
                            },
                            _ => error!("Received {:?} when expecting true or false for keyword terminals in assignment", keyword)
                        }
                    },
                    _ => error!("Received {:?} for terminal in assignment when expecting id, digit, char, or keyword", token)
                }
            },
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                debug!("Assignment nonterminal");
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
                self.add_code(0x8D);
                self.add_temp(static_offset);
                self.add_code(0x00);
            },
            _ => error!("Received {:?} when expecting terminal for assignmentchild in code gen", id_node)
        }
    }

    // Function for generating code for a print statement
    fn code_gen_print(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) {
        debug!("Code gen print statement");

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
                                debug!("Print id int");
                            },
                            Type::String => {
                                debug!("Print id string");
                            },
                            Type::Boolean => {
                                debug!("Print id boolean");
                            }
                        }
                    },
                    TokenType::Digit(digit) => {
                        // Sys call 1 for integers needs the number in Y
                        self.add_code(0xA0);
                        self.add_code(*digit as u8);

                        // And X = 1
                        self.add_code(0xA2);
                        self.add_code(0x01);
                    },
                    TokenType::Char(string) => {

                    },
                    TokenType::Keyword(keyword) => {

                    },
                    _ => error!("Received {:?} when expecting id, digit, string, or keyword for print terminal", token)
                }
            },
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                debug!("Print nonterminal");
            },
            _ => error!("Received {:?} when expecting terminal or AST nonterminal for print in code gen", child)
        }

        // The x and y registers are all set up, so just add the sys call
        self.add_code(0xFF);
    }
}
