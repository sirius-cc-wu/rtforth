extern crate rtforth;
use rtforth::core::VM;
use rtforth::tools::Tools;

#[test]
fn test_primitives() {
	let vm = &mut VM::new();
	vm.noop();
    vm.p_false();
    assert_eq!(vm.s_stack[0], 0);
    vm.p_true();
    assert_eq!(vm.s_stack[1], -1);
    vm.s_stack.push(2);
    assert_eq!(vm.s_stack.len(), 3);
    vm.dot_s();
    vm.words();
}

