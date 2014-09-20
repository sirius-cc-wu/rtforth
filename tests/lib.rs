extern crate jrforth;

#[test]
fn test_bye() {
	jrforth::vm::bye();
	assert!(true);
}