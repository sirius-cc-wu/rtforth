extern crate jrforth;
use jrforth::vm::VM;

#[test]
fn test_bye() {
	let vm = VM::new();
	VM::noop(&*vm);
	VM::quit(&*vm);
	VM::bye(&*vm);
	assert!(true);
}