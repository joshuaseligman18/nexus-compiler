use std::collections::HashMap;

use log::*;

use petgraph::graph::{NodeIndex, Graph};

use crate::util::nexus_log;

use web_sys::{Window, Document, Element, DomTokenList};

// Enum for determining the type of a variable in a symbol table
#[derive (Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    String,
    Boolean
}

// Enum for the symbol table entry fields to keep track of to prevent code duplication
#[derive (Debug)]
pub enum SymbolTableEntryField {
    Initialized,
    Used
}

// Basic struct for what needs to be stored for every symbol table entry
// id is excluded here because it is the key in the hashmap
#[derive (Debug)]
pub struct SymbolTableEntry {
    pub symbol_type: Type,
    pub position: (usize, usize),
    pub scope: usize,
    pub is_initialized: bool,
    pub is_used: bool
}

#[derive (Debug)]
pub struct SymbolTable {
    // The graph for the symbol table
    graph: Graph<HashMap<String, SymbolTableEntry>, ()>,

    // The index of the node of the current scope
    pub cur_scope: Option<usize>
}

impl SymbolTable {
    // Constructor for a new symbol table
    pub fn new() -> Self {
        return SymbolTable {
            graph: Graph::new(),
            cur_scope: None
        };
    }

    // Function to create a new scope and set it as the current scope
    pub fn new_scope(&mut self) {
        // Add a new node to the graph with the new hashmap
        let new_node: NodeIndex = self.graph.add_node(HashMap::new());
       
        // Check to see if we already have a scope
        if self.cur_scope.is_some() {
            // If so, then create the edge from the new scope to the parent
            self.graph.add_edge(new_node, NodeIndex::from(self.cur_scope.unwrap() as u32), ());
        }

        // Update the current scope to be the new scope
        self.cur_scope = Some(new_node.index());
    }

    // Function to manually set the current scope assuming it is in bounds
    // Used by the code generator
    pub fn set_cur_scope(&mut self, new_scope: usize) {
        self.cur_scope = Some(new_scope);
    }

    // Called to end the current  
    pub fn end_cur_scope(&mut self) {
        if self.cur_scope.is_some() {
            // Get a vector of neighbors
            let neighbors: Vec<NodeIndex> = self.graph.neighbors(NodeIndex::new(self.cur_scope.unwrap())).collect();

            if neighbors.len() > 0 {
                // Update the current scope to be the first in the list
                self.cur_scope = Some(neighbors[0].index());
            } else {
                // In the root scope and cur will be None now
                self.cur_scope = None;
            }
        }
    }

    // Adds an identifier to the current scope and returns if it was successful
    pub fn new_identifier(&mut self, id: String, id_type: Type, id_position: (usize, usize)) -> bool {
        // Get the current scope's hash table
        let scope_table: &mut HashMap<String, SymbolTableEntry> = self.graph.node_weight_mut(NodeIndex::new(self.cur_scope.unwrap())).unwrap();
        if (*scope_table).contains_key(&id) {
            // The id already exists so return false
            return false;
        } else {
            // Add the id and its respective information to the hash table
            let new_entry = SymbolTableEntry {
                symbol_type: id_type,
                position: id_position,
                scope: self.cur_scope.unwrap(),
                is_initialized: false,
                is_used: false
            };
            (*scope_table).insert(id, new_entry);
            return true;
        }
    }

    // Returns a reference to the appropriate symbol table entry
    // based on the current scope
    pub fn get_symbol(&mut self, id: &str) -> Option<&SymbolTableEntry> {
        // Start with the current scope
        let mut cur_scope_check: usize = self.cur_scope.unwrap();
      
        // This loop has checks at the end, but work has to be done first
        loop {
            // Get the hashmap for the scope
            let scope_table: &HashMap<String, SymbolTableEntry> = self.graph.node_weight(NodeIndex::new(cur_scope_check)).unwrap();
            if (*scope_table).contains_key(id) {
                // If the variable exists, then return the entry
                return (*scope_table).get(id);
            } else {
                if cur_scope_check == 0 {
                    // We are now in the master scope, so the variable does
                    // not exist relative to the current scope
                    return None;
                } else {
                    // Get a vector of neighbors
                    let neighbors: Vec<NodeIndex> = self.graph.neighbors(NodeIndex::new(cur_scope_check)).collect();
                    
                    // Move on the the next higher scope
                    cur_scope_check = neighbors[0].index();
                }
            }
        }
    }

    // Function to set a variable to be initialized
    pub fn set_entry_field(&mut self, id: &str, field: SymbolTableEntryField) {
        // Start with the current scope
        let mut cur_scope_use: usize = self.cur_scope.unwrap();

        loop {
            // Get the hashmap for the current scope being checked
            let scope_table: &mut HashMap<String, SymbolTableEntry> = self.graph.node_weight_mut(NodeIndex::new(cur_scope_use)).unwrap();
            if (*scope_table).contains_key(id) {
                // Get the entry and update the initialized field
                let id_entry: &mut SymbolTableEntry = (*scope_table).get_mut(id).unwrap();
                
                // Set the apprpriate flag based on the inputted field
                match field {
                    SymbolTableEntryField::Initialized => id_entry.is_initialized = true,
                    SymbolTableEntryField::Used => id_entry.is_used = true
                }
                break;
            } else {
                if cur_scope_use == 0 {
                    // Scope id of 0 means we are in the master scope, so break from the loop
                    break;
                } else {
                    // Move on to the next scope in the tree
                    let neighbors: Vec<NodeIndex> = self.graph.neighbors(NodeIndex::new(cur_scope_use)).collect();
                    cur_scope_use = neighbors[0].index();
                }
            }
        }
    }

    // Function to find all of the warnings after scope and type checks are completed
    pub fn mass_warnings(&mut self) -> i32 {
        let mut warning_count: i32 = 0;
        
        // Iterate through each scope
        for scope_table in self.graph.node_weights() {
            // Iterate through each entry in the scope's symbol table
            for (id_name, entry) in scope_table.iter() {
                if !entry.is_initialized {
                    if entry.is_used {
                        // Throw warning for declared and used but not initialized
                        nexus_log::log(
                            nexus_log::LogTypes::Warning,
                            nexus_log::LogSources::SemanticAnalyzer,
                            format!("Warning at {:?}; Id [ {} ] is declared and used, but never initialized", entry.position, id_name)
                        );
                        warning_count += 1;
                    } else {
                        // Throw warning for declared but never initialized or used
                        nexus_log::log(
                            nexus_log::LogTypes::Warning,
                            nexus_log::LogSources::SemanticAnalyzer,
                            format!("Warning at {:?}; Id [ {} ] is declared, but never initialized or used", entry.position, id_name)
                        );
                        warning_count += 1;
                    }
                } else {
                    if !entry.is_used {
                        // Throw warning for declared and initialized but never used
                        nexus_log::log(
                            nexus_log::LogTypes::Warning,
                            nexus_log::LogSources::SemanticAnalyzer,
                            format!("Warning at {:?}; Id [ {} ] is declared and initialized, but never used", entry.position, id_name)
                        );
                        warning_count += 1;
                    }
                }
            }
        }
        return warning_count;
    }

    pub fn display_symbol_table(&mut self, program_number: &u32) {
        self.initialize_symbol_table(program_number);
        self.populate_symbol_table(program_number);
    }

    fn initialize_symbol_table(&mut self, program_number: &u32) {
        // Get the preliminary objects
        let window: Window = web_sys::window().expect("Should be able to get the window");
        let document: Document = window.document().expect("Should be able to get the document");

        // Get the row element
        let symbol_table_area: Element = document.create_element("div").expect("Should be able to create the element");
        let symbol_table_area_classes: DomTokenList = symbol_table_area.class_list();
        symbol_table_area_classes.add_2("row", "symbol-table-area").expect("Should be able to add the classes");
        
        let symbol_table_elem: Element = document.create_element("table").expect("Should be able to create the table");
        let symbol_table_classes: DomTokenList = symbol_table_elem.class_list();
        symbol_table_classes.add_2("table", "table-striped").expect("Should be able to add the classes");
        symbol_table_elem.set_id(format!("program{}-symbol-table", *program_number).as_str());

        let symbol_table_head: Element = document.create_element("thead").expect("Should be able to create the element");
        let header_row: Element = document.create_element("tr").expect("Should be able to create the element");

        let id_head: Element = document.create_element("th").expect("Should be able to create the element");
        id_head.set_attribute("scope", "col").expect("Should be able to set the attribute");
        id_head.set_inner_html("Id");
        header_row.append_child(&id_head).expect("Should be able to add the child node");

        let type_head: Element = document.create_element("th").expect("Should be able to create the element");
        type_head.set_attribute("scope", "col").expect("Should be able to set the attribute");
        type_head.set_inner_html("Type");
        header_row.append_child(&type_head).expect("Should be able to add the child node");

        let scope_head: Element = document.create_element("th").expect("Should be able to create the element");
        scope_head.set_attribute("scope", "col").expect("Should be able to set the attribute");
        scope_head.set_inner_html("Scope");
        header_row.append_child(&scope_head).expect("Should be able to add the child node");

        let pos_head: Element = document.create_element("th").expect("Should be able to create the element");
        pos_head.set_attribute("scope", "col").expect("Should be able to set the attribute");
        pos_head.set_inner_html("Position");
        header_row.append_child(&pos_head).expect("Should be able to add the child node");

        let init_head: Element = document.create_element("th").expect("Should be able to create the element");
        init_head.set_attribute("scope", "col").expect("Should be able to set the attribute");
        init_head.set_inner_html("Init?");
        header_row.append_child(&init_head).expect("Should be able to add the child node");

        let used_head: Element = document.create_element("th").expect("Should be able to create the element");
        used_head.set_attribute("scope", "col").expect("Should be able to set the attribute");
        used_head.set_inner_html("Used?");
        header_row.append_child(&used_head).expect("Should be able to add the child node");

        symbol_table_head.append_child(&header_row).expect("Should be able to add the child node");
        symbol_table_elem.append_child(&symbol_table_head).expect("Should be able to add the child node");

        let symbol_body: Element = document.create_element("tbody").expect("Should be able to create the table body");
        symbol_body.set_id(format!("program{}-symbol-table-body", *program_number).as_str());
        symbol_table_elem.append_child(&symbol_body).expect("Should be able to add the child node");

        symbol_table_area.append_child(&symbol_table_elem).expect("Should be able to add the child node");

        let display_area: Element = document.get_element_by_id(format!("program{}-ast-pane", *program_number).as_str()).expect("Should be able to get element");
        display_area.append_child(&symbol_table_area).expect("Should be able to add child node");
    }

    // Function to populate the symbol table on the webpage
    fn populate_symbol_table(&mut self, program_number: &u32) {
         // Get the preliminary objects
        let window: Window = web_sys::window().expect("Should be able to get the window");
        let document: Document = window.document().expect("Should be able to get the document");

        let table_body: Element = document.get_element_by_id(format!("program{}-symbol-table-body", *program_number).as_str())
                                          .expect("Should be able to find the table body element");

        // Iterate through each scope
        for scope_table in self.graph.node_weights() {
            // Iterate through each entry in the scope's symbol table
            for (id_name, entry) in scope_table.iter() {
                let row_elem: Element = document.create_element("tr").expect("Should be able to create row element");

                let id_elem: Element = document.create_element("th").expect("Should be able to create id element");
                id_elem.set_inner_html(&id_name);
                id_elem.set_attribute("scope", "row").expect("Should be able to set the attribute");
                row_elem.append_child(&id_elem).expect("Should be able to append child node");

                let type_elem: Element = document.create_element("td").expect("Should be able to create type element");
                type_elem.set_inner_html(format!("{:?}", entry.symbol_type).as_str());
                row_elem.append_child(&type_elem).expect("Should be able to append child node");

                let scope_elem: Element = document.create_element("td").expect("Should be able to create scope element");
                scope_elem.set_inner_html(format!("{}", entry.scope).as_str());
                row_elem.append_child(&scope_elem).expect("Should be able to append child node");

                let position_elem: Element = document.create_element("td").expect("Should be able to create position element");
                position_elem.set_inner_html(format!("{:?}", entry.position).as_str());
                row_elem.append_child(&position_elem).expect("Should be able to append child node");

                let init_elem: Element = document.create_element("td").expect("Should be able to create init element");
                init_elem.set_inner_html(format!("{}", entry.is_initialized).as_str());
                row_elem.append_child(&init_elem).expect("Should be able to append child node");

                let used_elem: Element = document.create_element("td").expect("Should be able to create used element");
                used_elem.set_inner_html(format!("{}", entry.is_used).as_str());
                row_elem.append_child(&used_elem).expect("Should be able to append child node");

                table_body.append_child(&row_elem).expect("Should be ablo to append child node");
            }
        }
    }

    // Function to reset the symbol table for the new analysis
    pub fn reset(&mut self) {
        self.graph.clear();
        self.cur_scope = None;
    }
}
