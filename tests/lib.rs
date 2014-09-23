extern crate jrforth;
use jrforth::vm::VM;

#[test]
fn test_bye() {
	let vm = &mut VM::new();
	vm.find("");
	assert_eq!(0i, vm.found_index);
	VM::noop(vm);
	VM::quit(vm);
	VM::bye(vm);
	assert!(true);
}