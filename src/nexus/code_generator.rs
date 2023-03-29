use log::*;

use crate::nexus::{syntax_tree::SyntaxTree, syntax_tree_node::*, symbol_table::*};
use crate::nexus::token::{TokenType, Keywords};
use petgraph::graph::{NodeIndex};

use std::collections::HashMap;
use std::fmt;
use web_sys::{Document, Window, Element, DomTokenList};

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
    Data(u8)
}

// Customize the output when printing the string
impl fmt::Debug for CodeGenBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CodeGenBytes::Code(code) => write!(f, "{:02X}", code),
            CodeGenBytes::Var(var) => write!(f, "V{}", var),
            CodeGenBytes::Temp(temp) => write!(f, "T{}", temp),
            CodeGenBytes::Empty => write!(f, "00"),
            CodeGenBytes::Data(data) => write!(f, "{:02X}", data)
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
    string_history: HashMap<String, u8>
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

            static_table: HashMap::new(),

            // Always start with a temp index of 0
            temp_index: 0,

            string_history: HashMap::new()
        };

        // Initialize the entire array to be unused spot in memory
        for i in 0..0x100 {
            code_gen.code_arr.push(CodeGenBytes::Empty);
        }

        return code_gen;
    }

    pub fn generate_code(&mut self, ast: &SyntaxTree, symbol_table: &mut SymbolTable, program_number: &u32) {
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
        self.temp_index = 0;
        self.string_history.clear();

        // Generate the code for the program
        self.code_gen_block(ast, NodeIndex::new((*ast).root.unwrap()), symbol_table);
        // All programs end with 0x00, which is HALT
        self.add_code(0x00);
        debug!("{:?}", self.code_arr);

        self.backpatch_addresses();

        debug!("{:?}", self.static_table); 
        debug!("{:?}", self.code_arr);

        self.display_code(program_number);
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

    // Function to add byte of code to the memory array for variable addressing
    fn add_var(&mut self, var: usize) {
        // Add the code to the next available spot in memory
        self.code_arr[self.code_pointer as usize] = CodeGenBytes::Var(var);
        self.code_pointer += 1;
    }

    // Function to add byte of code to memory array for temporary data
    fn add_temp(&mut self, temp: usize) {
        // Add the addressing for the temporary value
        self.code_arr[self.code_pointer as usize] = CodeGenBytes::Temp(temp);
        self.code_pointer += 1;
    }

    // Function to add a byte of data to the heap
    fn add_data(&mut self, data: u8) {
        // Heap starts from the end of the 256 bytes and moves towards the front
        self.code_arr[self.heap_pointer as usize] = CodeGenBytes::Data(data);
        self.heap_pointer -= 1;
    }

    fn store_string(&mut self, string: &str) -> u8 {
        let addr: Option<&u8> = self.string_history.get(string);
        if addr.is_none() {
            // All strings are null terminated, so start with a 0x00 at the end
            self.add_data(0x00);

            // Loop through the string in reverse order
            for c in string.chars().rev() {
                // Add the ascii code of each character
                self.add_data(c as u8);
            }
            
            // Store it for future use
            self.string_history.insert(String::from(string), self.heap_pointer + 1);
            
            return self.heap_pointer + 1;
        } else {
            // The string is already on the heap, so return its address
            return *addr.unwrap();
        }
    }

    // Replaces temp addresses with the actual position in memory
    fn backpatch_addresses(&mut self) {
        for i in 0..self.code_arr.len() {
            match &self.code_arr[i] {
                CodeGenBytes::Var(offset) => {
                    self.code_arr[i] = CodeGenBytes::Code(self.code_pointer + *offset as u8);
                },
                CodeGenBytes::Temp(offset) => {
                    self.code_arr[i] = CodeGenBytes::Code(self.heap_pointer - *offset as u8);
                },
                _ => {}
            }
        }
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

                // Get the symbol table entry because strings have no code gen here, just the
                // static table entry
                let symbol_table_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap();
                match symbol_table_entry.symbol_type {
                    Type::Int | Type::Boolean => {
                        // Generate the code for the variable declaration
                        self.add_code(0xA9);
                        self.add_code(0x00);
                        self.add_code(0x8D);
                        self.add_var(static_offset);
                        self.add_code(0x00);
                    },
                    Type::String => { /* Nothing to do here */ }
                }
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
                        let value_id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 
                        let value_static_offset: usize = self.static_table.get(&(token.text.to_owned(), value_id_entry.scope)).unwrap().to_owned();
                        
                        self.add_code(0xAD);
                        self.add_var(value_static_offset);
                        self.add_code(0x00);
                    },
                    TokenType::Digit(val) => {
                        debug!("Assignment digit");
                        // Digits just load a constant to the accumulator
                        self.add_code(0xA9);
                        self.add_code(*val as u8);
                    },
                    TokenType::Char(string) => {
                        debug!("Assignment string");
                        
                        // Start by storing the string
                        let addr: u8 = self.store_string(&string);

                        // Store the starting address of the string in memory
                        self.add_code (0xA9);
                        self.add_code(addr);
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
                match non_terminal {
                    NonTerminalsAst::Add => {
                        // Call add, so the result will be in both the accumulator and in memory
                        self.code_gen_add(ast, children[0], symbol_table, true);
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
                self.add_code(0x8D);
                self.add_var(static_offset);
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
                        let static_offset: usize = self.static_table.get(&(id_name.to_owned(), print_id.scope)).unwrap().to_owned();
                        match &print_id.symbol_type {
                            Type::Int | Type::Boolean => {
                                debug!("Print id int/boolean");
                                
                                // Load the integer value into the Y register
                                self.add_code(0xAC);
                                self.add_var(static_offset);
                                self.add_code(0x00);

                                // Set X to 1 for the system call
                                self.add_code(0xA2);
                                self.add_code(0x01);
                            },
                            Type::String => {
                                debug!("Print id string");
                                // Store the string address in Y
                                self.add_code(0xAC);
                                self.add_var(static_offset);
                                self.add_code(0x00);

                                // X = 2 for this sys call
                                self.add_code(0xA2);
                                self.add_code(0x02);
                            },
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
                        // Store the string in memory and load its address to Y
                        let addr: u8 = self.store_string(&string);
                        self.add_code(0xA0);
                        self.add_code(addr);

                        // X = 2 for a string sys call
                        self.add_code(0xA2);
                        self.add_code(0x02);
                    },
                    TokenType::Keyword(keyword) => {
                        self.add_code(0xA0);
                        match keyword {
                            Keywords::True => {
                                // Y = 1 for true
                                self.add_code(0x01);
                            },
                            Keywords::False => {
                                // Y = 0 for false
                                self.add_code(0x00);
                            },
                            _ => error!("Received {:?} when expecting true or false for print keyword", keyword)
                        }
                        // X = 1 for the sys call
                        self.add_code(0xA2);
                        self.add_code(0x01);
                    },
                    _ => error!("Received {:?} when expecting id, digit, string, or keyword for print terminal", token)
                }
            },
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {
                debug!("Print nonterminal");
                match non_terminal {
                    NonTerminalsAst::Add => {
                        // Generate the result of the addition expression
                        self.code_gen_add(ast, children[0], symbol_table, true);

                        self.add_code(0x8D);
                        self.add_temp(self.temp_index);
                        self.temp_index += 1;
                        self.add_code(0x00);
                        
                        // Load the result to Y (wish there was TAY)
                        self.add_code(0xAC);
                        self.add_temp(self.temp_index - 1);
                        self.temp_index -= 1;
                        self.add_code(0x00);

                        // X = 1 for the sys call for integers
                        self.add_code(0xA2);
                        self.add_code(0x01);
                    },
                    _ => error!("Received {:?} when expecting addition or boolean expression for nonterminal print", non_terminal)
                }
            },
            _ => error!("Received {:?} when expecting terminal or AST nonterminal for print in code gen", child)
        }

        // The x and y registers are all set up, so just add the sys call
        self.add_code(0xFF);
    }

    // Function to generate code for an addition statement
    // Result is left in both the accumulator
    fn code_gen_add(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable, first: bool) {
        debug!("Code gen add");

        // Get the child for addition
        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
        let right_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[0]).unwrap();
        let left_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[1]).unwrap();

        match right_child {
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Digit(num) => {
                        // Store right side digit in the accumulator
                        self.add_code(0xA9);
                        self.add_code(*num);
                    },
                    TokenType::Identifier(id_name) => {
                        // Get the address needed from memory for the identifier
                        let value_id_entry: &SymbolTableEntry = symbol_table.get_symbol(&token.text).unwrap(); 
                        let value_static_offset: usize = self.static_table.get(&(token.text.to_owned(), value_id_entry.scope)).unwrap().to_owned();
                        
                        // Load the value into the accumulator
                        self.add_code(0xAD);
                        self.add_var(value_static_offset);
                        self.add_code(0x00);
                    },
                    _ => error!("Received {:?} when expecting digit or id for right side of addition", token)
                }

                // Both digits and ids are in the accumulator, so move them to
                // the res address for usage in the math operation
                self.add_code(0x8D);
                self.add_temp(self.temp_index);
                // We are using a new temporary value for temps, so increment the index
                self.temp_index += 1;
                self.add_code(0x00);
            },
            // Nonterminals are always add, so just call it
            SyntaxTreeNode::NonTerminalAst(non_terminal) => self.code_gen_add(ast, children[0], symbol_table, false),
            _ => error!("Received {:?} when expecting terminal or AST nonterminal for right addition value", right_child)
        }

        match left_child {
            SyntaxTreeNode::Terminal(token) => {
                match &token.token_type {
                    TokenType::Digit(num) => {
                        // Put left digit in acc
                        self.add_code(0xA9);
                        self.add_code(*num);

                        // Perform the addition
                        self.add_code(0x6D);
                        // Temp index - 1 is where the data is being stored
                        self.add_temp(self.temp_index - 1);
                        self.add_code(0x00);

                        // Only store the result back in memory if we have more addition to do
                        if !first {
                            // Store it back in the resulting address
                            self.add_code(0x8D);
                            self.add_temp(self.temp_index - 1);
                            self.add_code(0x00);
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
    }

    // Function to generate code for comparisons
    // Result is left in both the Z flag and the memory address that is returned,
    // so parent functions must clean up the memory that was used to prevent stack overflow
    fn code_gen_compare(&mut self, ast: &SyntaxTree, cur_index: NodeIndex, symbol_table: &mut SymbolTable, is_eq: bool) {
        debug!("Code gen add");

        // Get the child for comparison
        let children: Vec<NodeIndex> = (*ast).graph.neighbors(cur_index).collect();
        let right_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[0]).unwrap();
        let left_child: &SyntaxTreeNode = (*ast).graph.node_weight(children[1]).unwrap();

        // This is the address where the result is stored
        let mut res_addr: u8 = 0x00;

        match left_child {
            SyntaxTreeNode::Terminal(token) => {
                
            },
            SyntaxTreeNode::NonTerminalAst(non_terminal) => {

            },
            _ => error!("Received {:?} when expected terminal or AST nonterminal for left side of comparison in code gen", left_child)
        }
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
        display_area_class_list.add_2("container", "code-gen-pane").expect("Should be able to add the classes");

        // Get the array of values but only keep the hex digits and spaces
        let mut code_str: String = format!("{:?}", self.code_arr);
        code_str.retain(|c| c != ',' && c != '[' && c != ']');

        display_area_div.set_inner_html(&code_str);

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
