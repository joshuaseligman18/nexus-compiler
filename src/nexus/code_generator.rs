use log::*;

use crate::nexus::{syntax_tree::SyntaxTree, syntax_tree_node::*, symbol_table::SymbolTable};
use petgraph::graph::{NodeIndex};

use std::collections::HashMap;

#[derive (Debug)]
enum CodeGenBytes {
    // Representation for final code/data in memory
    Code(u8),
    // Temporary code until AST is traversed with identifier for later use
    Temp(usize),
    // Spot is available for anything to take it
    Empty
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

    // The static table
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

        self.code_gen_block(ast, NodeIndex::new((*ast).root.unwrap()), symbol_table);

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


    fn code_gen_var_decl(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable) {
        debug!("Code gen var decl");
        
        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
        let id_node: &SyntaxTreeNode = (*ast).graph.node_weight(children[0]).unwrap();

        match id_node {
            SyntaxTreeNode::Terminal(token) => {
                debug!("{:?}; {:?}", token.text, symbol_table.cur_scope.unwrap());
                let static_offset: usize = self.static_table.len();
                self.static_table.insert((token.text.to_owned(), symbol_table.cur_scope.unwrap()), static_offset);

                self.add_code(0xA9);
                self.add_code(0x00);
                self.add_temp(static_offset);
                self.add_code(0x00);
            },
            _ => error!("Received {:?} when expecting terminal for var decl child in code gen", id_node)
        }

    }
}
