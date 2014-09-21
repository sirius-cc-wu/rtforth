extern crate jrforth;
use jrforth::vm;

#[test]
fn test_bye() {
	let vm = vm::VM::new();
	vm.bye();
	assert!(true);
}