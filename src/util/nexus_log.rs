use wasm_bindgen::JsCast;
use web_sys::{HtmlTextAreaElement, Document, Window};

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
pub enum Sources {
    Nexus,
    Lexer
}

// Function that logs a message with the given type and source
pub fn log(log_type: LogTypes, src: Sources, msg: String) {
    // Get the log area
    let log_area: HtmlTextAreaElement = get_log_area();

    // Get the original value
    let mut log_value: String = log_area.value();

    // Add the new message to the logs
    log_value.push_str(format!("[{} - {}]: {}\n", log_type, src, msg).as_str());

    // Set the new value
    log_area.set_value(&log_value);
}

pub fn insert_empty_line() {
    // Get the log area
    let log_area: HtmlTextAreaElement = get_log_area();

    // Get the original value
    let mut log_value: String = log_area.value();

    // Add the new message to the logs
    log_value.push_str("\n");

    // Set the new value
    log_area.set_value(&log_value);
}

// Function to clean the logs
pub fn clear_logs() {
    // Get the log area
    let log_area: HtmlTextAreaElement = get_log_area();

    // Set the value to an empty string
    log_area.set_value("");
}

fn get_log_area() -> HtmlTextAreaElement {
    // Grab the window and document elements for DOM manipulation
    let window: Window = web_sys::window().expect("The window object should exist.");
    let document: Document = window.document().expect("The document object should exist");

    // Get the textarea where the logs are printed
    let log_area: HtmlTextAreaElement = document
        .get_element_by_id("nexus-log-area")
        .expect("There should be a nexus-log-area element")
        .dyn_into::<HtmlTextAreaElement>()
        .expect("The element should be recognized as a textarea");

    return log_area;
}