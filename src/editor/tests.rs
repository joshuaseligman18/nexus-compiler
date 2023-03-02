use log::debug;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Document, HtmlSelectElement, HtmlOptionElement, Window, Element};

use crate::util::test::*;

use wasm_bindgen::prelude::*;

// Have to import the editor js module
#[wasm_bindgen(module = "/editor.js")]
extern "C" {
    // Import the loadProgram function from js so we can call it from the Rust code
    #[wasm_bindgen(js_name = "loadProgram")]
    fn load_program(newCode: &str);
}


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
    add_test_button_fn(&load_test_btn)
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
fn add_test_button_fn(load_test_btn: &Element) {
    // Create a function that will be used as the event listener and add it to the load test button
    let load_test_fn: Closure<dyn FnMut()> = Closure::wrap(Box::new(move || {
        // Get the value to paste
        let test_value: String = get_current_test_value();
        // Paste the value
        load_program(&test_value);
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
            test_code: String::from("{\n  /* This is a COMMENT 007 */\n  string s\n  s = \"hello world\"\n  int a\n  a = 0\n  while (a != 5) {\n    a = 1 + a\n  }\n  if (a == 5) {\n    print(\"success\")\n  }\n  boolean b\n  b = true\n  if (b != false) {\n    print(s)\n  }\n}$")
        },
        Test {
            test_type: TestType::Lex,
            test_name: String::from("Everything but spaces"),
            test_code: String::from("{/* This is a COMMENT 007 */stringss=\"hello world\"intaa=0while(a!=5){a=1+a}if(a==5){print(\"success\")}booleanbb=trueif(b!=false){print(s)}}$")
        },
        Test {
            test_type: TestType::Lex,
            test_name: String::from("The pesky $"),
            test_code: String::from("{\n  /* This $ is in a comment and should do nothing.\n  The next $ should be the end of the program */\n}$\n  /* This $ should be an invalid character in the string */\n  print(\"hello $ world\")\n  /* A warning should be shown for not having the $ at the end of the program */\n}")
        },
        Test {
            test_type: TestType::Lex,
            test_name: String::from("Testing tabs"),
            test_code: String::from("{\n  /*\tTabs are only bad in strings.\n\tThey are ok as whitespace. */\n\tprint(\"testing\ttabs\")\n}$")
        },
        Test {
            test_type: TestType::Lex,
            test_name: String::from("Multi-line things"),
            test_code: String::from("{\n  /* This is a\n  multi-line comment */\n  string s\n  s = \"hello world\n  this should be throwing an error\"\n}$")
        },
        Test {
            test_type: TestType::Lex,
            test_name: String::from("Unclosed strings"),
            test_code: String::from("{\n  /* Unclosed string on the next line */\n  print(\"hi\n}$\n/* Unclosed string here too */ print(\"hi")
        },
        Test {
            test_type: TestType::Parse,
            test_name: String::from("Alan's tests"),
            test_code: String::from("{}$\n{{{{{{}}}}}}$\n{{{{{{}}} /* comments are ignored */ }}}}$\n{ /* comments are still ignored */ int @}$")
        },
        Test {
            test_type: TestType::Parse,
            test_name: String::from("Everything"),
            test_code: String::from("{\n  /* This is a COMMENT 007 */\n  string s\n  s = \"hello world\"\n  int a\n  a = 0\n  while (a != 5) {\n    a = 1 + a\n  }\n  if (a == 5) {\n    print(\"success\")\n  }\n  if true {\n    print(s)\n  }\n}$")
        },
        Test {
            test_type: TestType::Parse,
            test_name: String::from("Mismatched operation"),
            test_code: String::from("{\n  /* IntExpr = digit intop Expr, NOT Expr intop digit */\n  x = x + 3\n}$\n{\n  /* BoolExpr needs == or !=, not + */\n  while (true + false) {\n    print(\"no good\")\n  }\n}$\n{\n  /* Parentheses with a BoolExpr means comparison, not a single value */\n  while (true) {}\n}$")
        },
        Test {
            test_type: TestType::Parse,
            test_name: String::from("Mismatched types are ok"),
            test_code: String::from("{\n  /* Parse does not do type checking */\n  int x\n  x = 7 + \"james bond\"\n}$\n{\n  if (\"josh\" == 3) {\n    print(\"yay\")\n  }\n}$")
        },
        Test {
            test_type: TestType::Parse,
            test_name: String::from("Missing $"),
            test_code: String::from("{/* This should throw an error */}")
        },
        Test {
            test_type: TestType::Parse,
            test_name: String::from("Missing blocks"),
            test_code: String::from("{\n  if true print(\"hello\")\n}$\n{\n  int x\n  x = 2\n  while (x != 5) x = 1 + x\n}$\n/* Missing the block for the program */\nint a = 3")
        },
        Test {
            test_type: TestType::Parse,
            test_name: String::from("Empty things"),
            test_code: String::from("{/* Statement list is empty */}$\n{\n  /* Empty string should also compile */\n  string s\n  s = \"\"\n}$")
        },
        Test {
            test_type: TestType::Parse,
            test_name: String::from("Multi-digit numbers"),
            test_code: String::from("{\n  /* This should fail because assignments can only be 1 digit or an int operation */\n  int x\n  x = 42\n}$")
        },
        Test {
            test_type: TestType::Parse,
            test_name: String::from("Parser warnings"),
            test_code: String::from("{\n  /* Should have warnings for empty string and block */\n  s = \"\"\n  {}\n}$")
        },
        Test {
            test_type: TestType::Parse,
            test_name: String::from("End of file before end of program 1"),
            test_code: String::from("{  print(\"hello\"")
        },
        Test {
            test_type: TestType::Parse,
            test_name: String::from("End of file before end of program 2"),
            test_code: String::from("{  int a")
        },
        Test {
            test_type: TestType::Parse,
            test_name: String::from("End of file before end of program 3"),
            test_code: String::from("{ while")
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