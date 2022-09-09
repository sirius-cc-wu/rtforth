extern crate rtforth;
mod vm;

use rtforth::core::Core;
use rtforth::output::Output;
use std::process;
use vm::VM;

fn main() {
    let mut vm = VM::new(0x100);
    vm.add_primitive("bye", bye);

    vm.set_source(
        "
        : stars   2 activate  5 0 do pause 42 emit flush-output loop  nod ;
        : pluses   3 activate  5 0 do pause 43 emit flush-output loop  nod ;
        : main   stars  pluses  1000 ms  bye ;
    ",
    );
    vm.evaluate_input();
    if vm.last_error().is_some() {
        panic!("Error {:?} {:?}", vm.last_error().unwrap(), vm.last_token());
    }
    vm.flush_output();

    let main = vm.find("main").unwrap();
    vm.execute_word(main);
    vm.run();
}

/// Terminate process.
fn bye(vm: &mut VM) {
    vm.flush_output();
    process::exit(0);
}
