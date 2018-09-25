#[macro_use(primitive)]
extern crate rtforth;

use rtforth::core::Core;
use rtforth::output::Output;
use rtforth::vm::VM;
use std::process;

fn main() {
    let mut vm = VM::new(0x100, 0x100);
    vm.add_primitive("bye", bye);

    vm.set_source(
        "
        : nod   begin pause again ;
        : ms ( n -- )   mtime  begin mtime over -  2 pick <  while pause repeat  2drop ;
        : stars   1 activate  5 0 do 42 emit flush pause loop  nod ;
        : pluses   2 activate  5 0 do 43 emit flush pause loop  nod ;
        : main   stars  pluses  1000 ms  bye ;
    ",
    );
    vm.evaluate();
    match vm.last_error() {
        Some(e) => {
            println!("{}", e.description());
            vm.reset();
        }
        None => {}
    }

    let main = vm.find("main").unwrap();
    vm.execute_word(main);
    vm.run();
}

/// Terminate process.
primitive!{fn bye(vm: &mut VM) {
    vm.flush();
    process::exit(0);
}}
