use wasm_bindgen::{prelude::*, JsCast};
use log::{Level, info, debug};
use web_sys::{Window, Document, HtmlTextAreaElement, HtmlElement};

mod compiler;
use crate::compiler::Compiler;

#[wasm_bindgen]
pub fn nexus_init() {
    console_log::init_with_level(Level::Debug).expect("Should be able to connect to the browser's console");

    let window: Window = web_sys::window().expect("The window object should exist.");
    let document: Document = window.document().expect("The document object should exist");

    set_up_dom(&document);

    let nexus: Compiler = Compiler::new();

    info!("Nexus initialized");
}

fn set_up_dom(document: &Document) {
    let code_input: HtmlTextAreaElement = document
        .get_element_by_id("ta-code-input")
        .expect("There should be a ta-code-input element")
        .dyn_into::<HtmlTextAreaElement>()
        .expect("The element should be recognized as a textarea");
    
    let compile_btn: HtmlElement = document
        .get_element_by_id("compile-btn")
        .expect("There should be an element called compile-btn")
        .dyn_into::<HtmlElement>()
        .expect("Should be able to cast to an HtmlElement object");
    
    let submit_fn: Closure<dyn FnMut()> = Closure::<dyn FnMut()>::new(move || {
        debug!("Compile called");
    });
    compile_btn.add_event_listener_with_callback("click", submit_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    submit_fn.forget();
}