use log::debug;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Document, HtmlSelectElement, HtmlOptionElement, HtmlElement, HtmlTextAreaElement, Window, Element};

use crate::util::test::*;

// Function to set up the test suite
pub fn create_test_environment(document: &Document) {
    let test_options: HtmlSelectElement = document
        .get_element_by_id("tests")
        .expect("There should be a tests element")
        .dyn_into::<HtmlSelectElement>()
        .expect("The element should be recognized as a select element");

    // Grab the compile button
    let load_test_btn: Element = document
        .get_element_by_id("load-test-btn")
        .expect("There should be an element called load-test-btn");

    load_tests(document, &test_options);
    add_test_button_fn(document, &load_test_btn)
}

// Function to load the tests into the select element
fn load_tests(document: &Document, test_selection: &HtmlSelectElement) {
    // Get the tests
    let tests: Vec<Test> = get_tests();
    
    // Loop through all of the tests
    for test in tests.iter() {
        // Create the new option element with the given name and value
        let new_option = document
            .create_element("option")
            .expect("Should be able to create the option element")
            .dyn_into::<HtmlOptionElement>()
            .expect("Should be able to cast to option element");
        new_option.set_inner_text(format!("[{}] - {}", test.test_type, test.test_name).as_str());
        new_option.set_value(&test.test_code);

        // Add the option element to the dropdown menu
        test_selection.add_with_html_option_element(&new_option).expect("Should be able to add the option");
    }
}

// Function to set up the tests
fn add_test_button_fn(document: &Document, load_test_btn: &Element) {
    // Get the text area to paste the code into
    let code_input: HtmlTextAreaElement = document
        .get_element_by_id("ta-code-input")
        .expect("There should be a ta-code-input element")
        .dyn_into::<HtmlTextAreaElement>()
        .expect("The element should be recognized as a textarea");


        
    // Create a function that will be used as the event listener and add it to the load test button
    let load_test_fn: Closure<dyn FnMut()> = Closure::wrap(Box::new(move || {
        // Get the value to paste
        let test_value: String = get_current_test_value();
        // Paste the value
        code_input.set_value(&test_value);
    }) as Box<dyn FnMut()>);

    load_test_btn.add_event_listener_with_callback("click", load_test_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    load_test_fn.forget();
}

// Function that returns a vector of all of the tests
fn get_tests() -> Vec<Test> {
    let tests: Vec<Test> = vec![
        Test {
            test_type: TestType::Lex,
            test_name: String::from("Alan's tests"),
            test_code: String::from("{}$\n{{{{{{}}}}}}$\n{{{{{{}}} /* comments are ignored */ }}}}$\n{ /* comments are still ignored */ int @}$\n{\nint a\na = a\nstring b\na=b\n}$")
        },
        Test {
            test_type: TestType::Lex,
            test_name: String::from("Everything"),
            test_code: String::from("{\n  /* This is a COMMENT 007 */\n  string s\n  s = \"hello world\"\n  int a\n  a = 0\n  while (a != 5) {\n    a = a + 1\n  }\n  if (a == 5) {\n    print(\"success\")\n  }\n  boolean b\n  b = true\n  if (b != false) {\n    print(s)\n  }\n}$")
        },
        Test {
            test_type: TestType::Lex,
            test_name: String::from("Everything but spaces"),
            test_code: String::from("{/* This is a COMMENT 007 */stringss=\"hello world\"intaa=0while(a!=5){a=a+1}if(a==5){print(\"success\")}booleanbb=trueif(b!=false){print(s)}}$")
        },
        Test {
            test_type: TestType::Lex,
            test_name: String::from("The pesky $"),
            test_code: String::from("{\n  /* This $ is in a comment and should do nothing.\n  The next $ should be the end of the program */\n}$\n  /* This $ should be an invalid character in the string */\n  print(\"hello $ world\")\n  /* A warning should be shown for not having the $ at the end of the program */\n}")
        },
        Test {
            test_type: TestType::Lex,
            test_name: String::from("Testing tabs"),
            test_code: String::from("{\n  /*\tThis tab should be ok since it is in a comment.\n  \tThe others should throw errors. */\n\tprint(\"testing\ttabs\")\n}$")
        }
    ];

    return tests;
}

// Function to get the current test
fn get_current_test_value() -> String {
    // Grab the window and document elements for DOM manipulation
    let window: Window = web_sys::window().expect("The window object should exist");
    let document: Document = window.document().expect("The document object should exist");

    // Get the select element and return its value
    let test_options: HtmlSelectElement = document
        .get_element_by_id("tests")
        .expect("There should be a tests element")
        .dyn_into::<HtmlSelectElement>()
        .expect("The element should be recognized as a select element");
    
    return test_options.value();
}