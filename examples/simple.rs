extern crate rtforth;
use rtforth::vm::VM;
use rtforth::core::Core;

// Evaluate "1 ."
fn main() {
    let vm = &mut VM::new(100, 100);
    vm.set_source("1 .");
    vm.evaluate();
    match vm.last_error() {
        Some(e) => println!("{:?}", e),
        None => {}
    }
}
