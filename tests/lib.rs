extern crate jrforth;
use jrforth::bye;

#[test]
fn test_bye() {
	bye();
	assert!(true);
}