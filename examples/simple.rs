extern crate rtforth;
use rtforth::vm::VM;
use rtforth::core::Core;
use rtforth::output::Output;

// Evaluate "1 ."
fn main() {
    let vm = &mut VM::new(65536);
    vm.add_core();
    vm.add_output();
    vm.set_source("1 .");
    match vm.evaluate() {
        Err(e) => println!("{:?}", e),
        Ok(()) => {}
    }
}
