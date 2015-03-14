extern crate jrforth;
use jrforth::vm::VM;

#[test]
fn test_find() {
	let vm = &mut VM::new();
	vm.find("");
	assert_eq!(0isize, vm.found_index);
	vm.find("word-not-exist");
	assert_eq!(0isize, vm.found_index);
	vm.find("noop");
	assert_eq!(1isize, vm.found_index);
}

#[test]
fn test_primitives() {
	let vm = &mut VM::new();
	vm.noop();
    vm.s_stack.push(1);
    vm.dot_s();
	vm.quit();
	assert!(true);
}

#[test]
fn test_inner_interpreter_without_nest () {
	let vm = &mut VM::new();
    vm.find("noop");
    let idx = vm.found_index;
    vm.compile_word(idx);
    vm.compile_integer(3);
    vm.compile_integer(2);
    vm.compile_integer(1);
    vm.inner_interpret(1);
    assert_eq!(3usize, vm.s_stack.len());
}

