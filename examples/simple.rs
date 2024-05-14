extern crate rtforth;
mod vm;

use rtforth::core::Core;
use vm::VM;

// Evaluate "1 ."
fn main() {
    let vm = &mut VM::new(100);
    vm.set_source("1 . flush-output");
    vm.evaluate_input();
    match vm.last_error() {
        Some(e) => println!("{:?}", e),
        None => {}
    }
}
