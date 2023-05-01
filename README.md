# nexus-compiler
Nexus is a compiler written in Rust (and a little JavaScript) for a custom language grammar that runs in the browser with WebAssembly. Since WebAssembly is a relatively new technology, it is recommended to use the lastest version of Firefox or Chrome to ensure compatibility. The custom language grammar that Nexus compiles for can be found [here](https://www.labouseur.com/courses/compilers/grammar.pdf). Nexus targets a subset of the 6502 instruction set, which can be found [here](https://www.labouseur.com/commondocs/6502alan-instruction-set.pdf), as well as RISC-V assembly (see *riscv-resources/resources.md* for more information). 

## Project Locations
* Project 1 (Lexer) - project1 branch
* Project 2 (Parser) - project2 branch
* Project 3 (Semantic Analyzer) - project3 branch
* Project 4 (Code Generator) - main and project4 branch

## Setup Instructions
1. Install Rust, which can be found [here](https://www.rust-lang.org/tools/install).
2. Make sure `cargo` is added to your $PATH environment variable. Its default location is ~/.cargo/bin.
3. Install `wasm-pack` to get all of the WebAssembly utilities for compiling the Rust code by running `cargo install wasm-pack`.
4. Install Node.js, which can be found [here](https://nodejs.org/en/).
5. Make sure Node.js is added to your $PATH environment variable. Its default location is ~/.npm-global/bin.
6. *Recommended:* Install `serve` to be able to start up a simple web server by running `npm i -g serve`. If this is not done, the `make run` command will automatically call `npx serve` to collect the package from the internet.

## Nexus Makefile Commands
* `make` / `make build`: Builds Nexus into a WebAssembly module that can be run on the web through JavaScript.
* `make clean`: Removes the files created when the project is built, including the WebAssembly output.
* `make run`: Spins up a basic server to host Nexus. This is required as the current state of WebAssembly requires it to be fetched and it cannot be directly imported to the JavaScript.
* Alan: Run in Chrome.

## RISC-V Execution Instructions
* Install the RISC-V GNU Toolchain, which can be found [here](https://github.com/riscv-software-src/homebrew-riscv).
* Compile your program in Nexus with RISC-V target selected.
* Copy/paste the output assembly into a file called *my_program.s*. The "Lots of loops" program output is provided in *riscv-resources/my_program.s*.
* `make` the assembly using the provided *makefile* in the *riscv-resources* folder.
* `make run` to execute the program.
* *Note: GDB is really buggy for the RISC-V tools, so it is recommended to just run the program without debug options.*
