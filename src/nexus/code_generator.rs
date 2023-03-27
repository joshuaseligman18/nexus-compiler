use log::*;

use crate::nexus::{syntax_tree::SyntaxTree, symbol_table::SymbolTable};
use petgraph::graph::{NodeIndex};

pub struct CodeGenerator {
    max_scope: usize
}

impl CodeGenerator {
    pub fn new() -> Self {
        return CodeGenerator {
            // This is a flag for a new program
            max_scope: usize::MAX
        };
    }

    pub fn generate_code(&mut self, ast: &SyntaxTree, symbol_table: &mut SymbolTable) {
        debug!("Code gen called");

        // Make sure the current scope is set to be a flag for none
        self.max_scope = usize::MAX;
        self.code_gen_block(ast, (*ast).root.unwrap(), symbol_table);
    }

    fn code_gen_block(&mut self, ast: &SyntaxTree, cur_index: usize, symbol_table: &mut SymbolTable) {
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
        let neighbors: Vec<NodeIndex> = (*ast).graph.neighbors(NodeIndex::new(cur_index)).collect();

        for neighbor_index in neighbors.into_iter().rev() {
            debug!("{:?}", (*ast).graph.node_weight(neighbor_index).unwrap());
        }

        // Exit the current scope
        symbol_table.end_cur_scope();
    }
}
