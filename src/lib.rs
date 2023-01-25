use wasm_bindgen::prelude::*;
use log::{Level, info, debug};
use web_sys::{Window, Document};

mod nexus;
mod util;
mod editor;

use editor::*;

// Function to initialize Nexus
#[wasm_bindgen]
pub fn nexus_init() {
    // Set up console logs for debugging
    console_log::init_with_level(Level::Debug).expect("Should be able to connect to the browser's console");
    console_error_panic_hook::set_once();

    // Grab the window and document elements for DOM manipulation
    let window: Window = web_sys::window().expect("The window object should exist");
    let document: Document = window.document().expect("The document object should exist");

    // Set up the event listeners
    buttons::set_up_buttons(&document);
    tests::create_test_environment(&document);

    info!("Nexus initialized");
}