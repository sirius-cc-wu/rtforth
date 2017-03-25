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
    vm.evaluate();
    match vm.last_error() {
        Some(e) => println!("{:?}", e),
        None => {}
    }
}
