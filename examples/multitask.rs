#[macro_use(primitive)]
extern crate rtforth;
mod vm;

use rtforth::core::Core;
use rtforth::exception;
use rtforth::output::Output;
use std::process;
use vm::VM;

fn main() {
    let mut vm = VM::new(400 * 1024);
    vm.add_primitive("bye", bye);

    vm.set_source(
        "
        : stars   2 activate  5 0 do pause 42 emit flush-output loop  nod ;
        : pluses   3 activate  5 0 do pause 43 emit flush-output loop  nod ;
        : main   stars  pluses  1000 ms  bye ;
    ",
    );
    vm.evaluate_input();
    match vm.last_error() {
        Some(e) => {
            println!("{}", exception::description(e));
            vm.reset();
        }
        None => {}
    }

    let main = vm.find("main").unwrap();
    vm.execute_word(main);
    vm.run();
}

// Terminate process.
primitive! {fn bye(vm: &mut VM) {
    vm.flush_output();
    process::exit(0);
}}
