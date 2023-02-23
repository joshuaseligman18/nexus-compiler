use wasm_bindgen::JsCast;
use web_sys::{HtmlTextAreaElement, Document, Window, Element, DomTokenList};

// Defines the type of logs
// https://stackoverflow.com/questions/69015213/how-can-i-display-an-enum-in-lowercase
#[derive (Debug, strum::Display)]
#[strum (serialize_all = "UPPERCASE")]
pub enum LogTypes {
    Info,
    Warning,
    Error,
    Debug
}

// Defines where the logs can come from
#[derive (Debug, strum::Display)]
#[strum (serialize_all = "UPPERCASE")]
pub enum LogSources {
    Nexus,
    Lexer,
    Parser,
    SemanticAnalyzer,
    CodeGenerator
}

// Function that logs a message with the given type and source
pub fn log(log_type: LogTypes, src: LogSources, msg: String) {
    // Get the log area
    let log_area: Element = get_log_area();

    // Create the new element to place in the logs
    let new_log: Element = get_document().create_element("p").expect("Should be able to create the element");
    new_log.set_inner_html(format!("[{} - {}]: {}", log_type, src, msg).as_str());

    // Set the new value
    log_area.append_child(&new_log).expect("Should be able to add the child");

    // Special cases and such
    match log_type {
        LogTypes::Debug => {
            // Only log if in verbose mode
            if !is_verbose_mode(&src) {
                log_area.remove_child(&new_log).expect("Should be able to remove the child");
            }
        },
        LogTypes::Error => {
            // Errors have special classes
            new_log.set_class_name("error");
        },
        LogTypes::Warning => {
            // Set the warning class
            new_log.set_class_name("warning");
        },
        _ => {
            // Nothing else to do here
        }
    }
}

pub fn insert_empty_line() {
    // Get the log area
    let log_area: Element = get_log_area();

    // The new line is just a br tag
    let new_line: Element = get_document().create_element("br").expect("Should be able to create the br element");
    log_area.append_child(&new_line).expect("Should be able to add the child");
}

// Function to clean the logs
pub fn clear_logs() {
    // Get the log area
    let log_area: Element = get_log_area();

    // Remove all children by wiping the inner html
    log_area.set_inner_html("");
}

fn get_log_area() -> Element {
    let document: Document = get_document();

    // Get the area where the logs are printed
    let log_area: Element = document
        .get_element_by_id("nexus-log-area")
        .expect("There should be a nexus-log-area element");

    return log_area;
}

fn get_document() -> Document {
    // Grab the window and document elements for DOM manipulation
    let window: Window = web_sys::window().expect("The window object should exist.");
    let document: Document = window.document().expect("The document object should exist");

    return document;
}

fn is_verbose_mode(src: &LogSources) -> bool {
    // Grab the window and document elements for DOM manipulation
    let window: Window = web_sys::window().expect("The window object should exist.");
    let document: Document = window.document().expect("The document object should exist");

    // Assume we are in verbose mode
    let mut out: bool = true;

    // Get the target button element
    let target: Element = match src {
        LogSources::Nexus => document.get_element_by_id("nexus-log-mode").expect("Should be able to find the nexus-log-mode element"),
        LogSources::Lexer => document.get_element_by_id("lexer-log-mode").expect("Should be able to find the lexer-log-mode element"),
        LogSources::Parser => document.get_element_by_id("parser-log-mode").expect("Should be able to find the parser-log-mode element"),
        LogSources::SemanticAnalyzer => document.get_element_by_id("semantic-log-mode").expect("Should be able to find the semantic-log-mode element"),
        LogSources::CodeGenerator => document.get_element_by_id("codegen-log-mode").expect("Should be able to find the codegen-log-mode element"),
    };

    // Check to see if it is in simple mode
    let class_list: DomTokenList = target.class_list();
    if class_list.contains("simple") {
        out = false;
    }
    return out;
}