//! Exception constants

use std::convert::From;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Exception(i16);

impl From<Exception> for isize {
    fn from(e: Exception) -> Self {
        e.0 as _
    }
}

/// = -1, ANS Forth
pub const ABORT: Exception = Exception(-1);
/// = -2, ANS Forth
pub const ABORT_QUOTE: Exception = Exception(-2);
/// = -3, ANS Forth
pub const STACK_OVERFLOW: Exception = Exception(-3);
/// = -4, ANS Forth
pub const STACK_UNDERFLOW: Exception = Exception(-4);
/// = -5, ANS Forth
pub const RETURN_STACK_OVERFLOW: Exception = Exception(-5);
/// = -6, ANS Forth
pub const RETURN_STACK_UNDERFLOW: Exception = Exception(-6);
/// = -7, ANS Forth
pub const DO_LOOP_NESTED_TOO_DEEPLY: Exception = Exception(-7);
/// = -8, ANS Forth
pub const DICTIONARY_OVERFLOW: Exception = Exception(-8);
/// = -9, ANS Forth
pub const INVALID_MEMORY_ADDRESS: Exception = Exception(-9);
/// = -10, ANS Forth
pub const DIVISION_BY_ZERO: Exception = Exception(-10);
/// = -11, ANS Forth
pub const RESULT_OUT_OF_RANGE: Exception = Exception(-11);
/// = -12, ANS Forth
pub const ARGUMENT_TYPE_MISMATCH: Exception = Exception(-12);
/// = -13, ANS Forth
pub const UNDEFINED_WORD: Exception = Exception(-13);
/// = -14, ANS Forth
pub const INTERPRETING_A_COMPILE_ONLY_WORD: Exception = Exception(-14);
/// = -15, ANS Forth
pub const INVALID_FORGET: Exception = Exception(-15);
/// = -16, ANS Forth
pub const ATTEMPT_TO_USE_ZERO_LENGTH_STRING: Exception = Exception(-16);
/// = -17, ANS Forth
pub const PICTURED_NUMERIC_OUTPUT_STRING_OVERFLOW: Exception = Exception(-17);
/// = -18, ANS Forth
pub const PARSED_STRING_OVERFLOW: Exception = Exception(-18);
/// = -19, ANS Forth
pub const DEFINITION_NAME_TOO_LONG: Exception = Exception(-19);
/// = -20, ANS Forth
pub const WRITE_TO_A_READ_ONLY_LOCATION: Exception = Exception(-20);
/// = -21, ANS Forth
pub const UNSUPPORTED_OPERATION: Exception = Exception(-21);
/// = -22, ANS Forth
pub const CONTROL_STRUCTURE_MISMATCH: Exception = Exception(-22);
/// = -23, ANS Forth
pub const ADDRESS_ALIGNMENT_EXCEPTION: Exception = Exception(-23);
/// = -24, ANS Forth
pub const INVALID_NUMERIC_ARGUMENT: Exception = Exception(-24);
/// = -25, ANS Forth
pub const RETURN_STACK_IMBALANCE: Exception = Exception(-25);
/// = -26, ANS Forth
pub const LOOP_PARAMETERS_UNAVAILABLE: Exception = Exception(-26);
/// = -27, ANS Forth
pub const INVALID_RECURSION: Exception = Exception(-27);
/// = -28, ANS Forth
pub const USER_INTERRUPT: Exception = Exception(-28);
/// = -29, ANS Forth
pub const COMPILER_NESTING: Exception = Exception(-29);
/// = -30, ANS Forth
pub const OBSOLESCENT_FEATURE: Exception = Exception(-30);
/// = -31, ANS Forth
pub const TO_BODY_USED_ON_NON_CREATED_DEFINITION: Exception = Exception(-31);
/// = -32, ANS Forth
pub const INVALID_NAME_ARGUMENT: Exception = Exception(-32);
/// = -33, ANS Forth
pub const BLOCK_READ_EXCEPTION: Exception = Exception(-33);
/// = -34, ANS Forth
pub const BLOCK_WRITE_EXCEPTION: Exception = Exception(-34);
/// = -35, ANS Forth
pub const INVALID_BLOCK_NUMBER: Exception = Exception(-35);
/// = -36, ANS Forth
pub const INVALID_FILE_POSITION: Exception = Exception(-36);
/// = -37, ANS Forth
pub const FILE_IO_EXCEPTION: Exception = Exception(-37);
/// = -38, ANS Forth
pub const NON_EXISTENT_FILE: Exception = Exception(-38);
/// = -39, ANS Forth
pub const UNEXPECTED_END_OF_FILE: Exception = Exception(-39);
/// = -40, ANS Forth
pub const INVALID_BASE_FOR_FLOATING_POINT_CONVERSION: Exception = Exception(-40);
/// = -41, ANS Forth
pub const LOSS_OF_PRECISION: Exception = Exception(-41);
/// = -42, ANS Forth
pub const FLOATING_POINT_DIVIDED_BY_ZERO: Exception = Exception(-42);
/// = -43, ANS Forth
pub const FLOATING_POINT_RESULT_OUT_OF_RANGE: Exception = Exception(-43);
/// = -44, ANS Forth
pub const FLOATING_POINT_STACK_OVERFLOW: Exception = Exception(-44);
/// = -45, ANS Forth
pub const FLOATING_POINT_STACK_UNDERFLOW: Exception = Exception(-45);
/// = -46, ANS Forth
pub const FLOATING_POINT_INVALID_ARGUMENT: Exception = Exception(-46);
/// = -47, ANS Forth
pub const COMPILATION_WORD_LIST_DELETED: Exception = Exception(-47);
/// = -48, ANS Forth
pub const INVALID_POSTPONE: Exception = Exception(-48);
/// = -49, ANS Forth
pub const SEARCH_ORDER_OVERFLOW: Exception = Exception(-49);
/// = -50, ANS Forth
pub const SEARCH_ORDER_UNDERFLOW: Exception = Exception(-50);
/// = -51, ANS Forth
pub const COMPILATION_WORD_LIST_CHANGED: Exception = Exception(-51);
/// = -52, ANS Forth
pub const CONTROL_FLOW_STACK_OVERFLOW: Exception = Exception(-52);
/// = -53, ANS Forth
pub const EXCEPTION_STACK_OVERFLOW: Exception = Exception(-53);
/// = -54, ANS Forth
pub const FLOATING_POINT_UNDERFLOW: Exception = Exception(-54);
/// = -55, ANS Forth
pub const FLOATING_POINT_UNIDENTIFIED_FAULT: Exception = Exception(-55);
/// = -56, ANS Forth
pub const QUIT: Exception = Exception(-56);
/// = -57, ANS Forth
pub const EXCEPTION_IN_SENDING_OR_RECEIVING_A_CHARACTER: Exception = Exception(-57);
/// = -58, ANS Forth
pub const BRACKET_IF_ELSE_OR_THEN_EXCEPTION: Exception = Exception(-58);

/// Description of the exception
pub fn description(e: Exception) -> &'static str {
    match e {
        ABORT => "Aborted",
        ABORT_QUOTE => "Aborted",
        STACK_OVERFLOW => "Stack overflow",
        STACK_UNDERFLOW => "Stack underflow",
        RETURN_STACK_OVERFLOW => "Return stack overflow",
        RETURN_STACK_UNDERFLOW => "Return stack underflow",
        DO_LOOP_NESTED_TOO_DEEPLY => "Do-loop nested too deeply",
        DICTIONARY_OVERFLOW => "Dictionary overflow",
        INVALID_MEMORY_ADDRESS => "Invalid memory address",
        DIVISION_BY_ZERO => "Division by zero",
        RESULT_OUT_OF_RANGE => "Result out of range",
        ARGUMENT_TYPE_MISMATCH => "Argument type mismatch",
        UNDEFINED_WORD => "Undefined word",
        INTERPRETING_A_COMPILE_ONLY_WORD => "Interpreting a compile only word",
        INVALID_FORGET => "Invalid FORGET",
        ATTEMPT_TO_USE_ZERO_LENGTH_STRING => "Attempt to use zero length string",
        PICTURED_NUMERIC_OUTPUT_STRING_OVERFLOW => "Picture numeric output string overflow",
        PARSED_STRING_OVERFLOW => "Parsed string overflow",
        DEFINITION_NAME_TOO_LONG => "Definition name too long",
        WRITE_TO_A_READ_ONLY_LOCATION => "Write to a read only location",
        UNSUPPORTED_OPERATION => "Unsupported operation",
        CONTROL_STRUCTURE_MISMATCH => "Control structure mismatch",
        ADDRESS_ALIGNMENT_EXCEPTION => "Address alignment exception",
        INVALID_NUMERIC_ARGUMENT => "Invalid numeric argument",
        RETURN_STACK_IMBALANCE => "Return stack imbalance",
        LOOP_PARAMETERS_UNAVAILABLE => "Loop parameters unavailable",
        INVALID_RECURSION => "Invalid recursion",
        USER_INTERRUPT => "User interrupt",
        COMPILER_NESTING => "Compiler nesting",
        OBSOLESCENT_FEATURE => "Obsolescent feature",
        TO_BODY_USED_ON_NON_CREATED_DEFINITION => ">BODY used on non-CREATEd definition",
        INVALID_NAME_ARGUMENT => "Invalid name argument",
        BLOCK_READ_EXCEPTION => "Block read exception",
        BLOCK_WRITE_EXCEPTION => "Block write exception",
        INVALID_BLOCK_NUMBER => "Invalid block number",
        INVALID_FILE_POSITION => "Invalid file position",
        FILE_IO_EXCEPTION => "File I/O exception",
        NON_EXISTENT_FILE => "Non-existent file",
        UNEXPECTED_END_OF_FILE => "Unexpected end of file",
        INVALID_BASE_FOR_FLOATING_POINT_CONVERSION => "Invalid BASE for floating point conversion",
        LOSS_OF_PRECISION => "Loss of precision",
        FLOATING_POINT_DIVIDED_BY_ZERO => "Floating point divided by zero",
        FLOATING_POINT_RESULT_OUT_OF_RANGE => "Floating point result out of range",
        FLOATING_POINT_STACK_OVERFLOW => "Floating point stack overflow",
        FLOATING_POINT_STACK_UNDERFLOW => "Floating point stack underflow",
        FLOATING_POINT_INVALID_ARGUMENT => "Floating point invalid argument",
        COMPILATION_WORD_LIST_DELETED => "Compilation word list deleted",
        INVALID_POSTPONE => "Invalid POSTPONE",
        SEARCH_ORDER_OVERFLOW => "Search order overflow",
        SEARCH_ORDER_UNDERFLOW => "Search order underflow",
        COMPILATION_WORD_LIST_CHANGED => "Compilation word list changed",
        CONTROL_FLOW_STACK_OVERFLOW => "Control flow stack overflow",
        EXCEPTION_STACK_OVERFLOW => "Exception stack overflow",
        FLOATING_POINT_UNDERFLOW => "Floating point underflow",
        FLOATING_POINT_UNIDENTIFIED_FAULT => "Floating point unidentified fault",
        QUIT => "QUIT",
        EXCEPTION_IN_SENDING_OR_RECEIVING_A_CHARACTER => {
            "Exception in sending or receiving a character"
        }
        BRACKET_IF_ELSE_OR_THEN_EXCEPTION => "[IF],[ELSE],[THEN] exception",
        _ => "",
    }
}
