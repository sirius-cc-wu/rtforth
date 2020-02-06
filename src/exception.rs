
/// Exceptions assigned by Forth standard

pub const ABORT: isize = -1;
pub const ABORT_QUOTE: isize = -2;
pub const STACK_OVERFLOW: isize = -3;
pub const STACK_UNDERFLOW: isize = -4;
pub const RETURN_STACK_OVERFLOW: isize = -5;
pub const RETURN_STACK_UNDERFLOW: isize = -6;
pub const DO_LOOP_NESTED_TOO_DEEPLY: isize = -7;
pub const DICTIONARY_OVERFLOW: isize = -8;
pub const INVALID_MEMORY_ADDRESS: isize = -9;
pub const DIVISION_BY_ZERO: isize = -10;
pub const RESULT_OUT_OF_RANGE: isize = -11;
pub const ARGUMENT_TYPE_MISMATCH: isize = -12;
pub const UNDEFINED_WORD: isize = -13;
pub const INTERPRETING_A_COMPILE_ONLY_WORD: isize = -14;
pub const INVALID_FORGET: isize = -15;
pub const ATTEMPT_TO_USE_ZERO_LENGTH_STRING: isize = -16;
pub const PICTURED_NUMERIC_OUTPUT_STRING_OVERFLOW: isize = -17;
pub const PARSED_STRING_OVERFLOW: isize = -18;
pub const DEFINITION_NAME_TOO_LONG: isize = -19;
pub const WRITE_TO_A_READ_ONLY_LOCATION: isize = -20;
pub const UNSUPPORTED_OPERATION: isize = -21;
pub const CONTROL_STRUCTURE_MISMATCH: isize = -22;
pub const ADDRESS_ALIGNMENT_EXCEPTION: isize = -23;
pub const INVALID_NUMERIC_ARGUMENT: isize = -24;
pub const RETURN_STACK_IMBALANCE: isize = -25;
pub const LOOP_PARAMETERS_UNAVAILABLE: isize = -26;
pub const INVALID_RECURSION: isize = -27;
pub const USER_INTERRUPT: isize = -28;
pub const COMPILER_NESTING: isize = -29;
pub const OBSOLESCENT_FEATURE: isize = -30;
pub const TO_BODY_USED_ON_NON_CREATED_DEFINITION: isize = -31;
pub const INVALID_NAME_ARGUMENT: isize = -32;
pub const BLOCK_READ_EXCEPTION: isize = -33;
pub const BLOCK_WRITE_EXCEPTION: isize = -34;
pub const INVALID_BLOCK_NUMBER: isize = -35;
pub const INVALID_FILE_POSITION: isize = -36;
pub const FILE_IO_EXCEPTION: isize = -37;
pub const NON_EXISTENT_FILE: isize = -38;
pub const UNEXPECTED_END_OF_FILE: isize = -39;
pub const INVALID_BASE_FOR_FLOATING_POINT_CONVERSION: isize = -40;
pub const LOSS_OF_PRECISION: isize = -41;
pub const FLOATING_POINT_DIVIDED_BY_ZERO: isize = -42;
pub const FLOATING_POINT_RESULT_OUT_OF_RANGE: isize = -43;
pub const FLOATING_POINT_STACK_OVERFLOW: isize = -44;
pub const FLOATING_POINT_STACK_UNDERFLOW: isize = -45;
pub const FLOATING_POINT_INVALID_ARGUMENT: isize = -46;
pub const COMPILATION_WORD_LIST_DELETED: isize = -47;
pub const INVALID_POSTPONE: isize = -48;
pub const SEARCH_ORDER_OVERFLOW: isize = -49;
pub const SEARCH_ORDER_UNDERFLOW: isize = -50;
pub const COMPILATION_WORD_LIST_CHANGED: isize = -51;
pub const CONTROL_FLOW_STACK_OVERFLOW: isize = -52;
pub const EXCEPTION_STACK_OVERFLOW: isize = -53;
pub const FLOATING_POINT_UNDERFLOW: isize = -54;
pub const FLOATING_POINT_UNIDENTIFIED_FAULT: isize = -55;
pub const QUIT: isize = -56;
pub const EXCEPTION_IN_SENDING_OR_RECEIVING_A_CHARACTER: isize = -57;
pub const BRACKET_IF_ELSE_OR_THEN_EXCEPTION: isize = -58;

/// rtForth system exceptions (-511..-256)

pub const UNSUPPORTED_BASE_FOR_INTEGER_CONVERSION: isize = -256;
pub const INVALID_EXECUTION_TOKEN: isize = -257;
pub const INTEGER_UNIDENTIFIED_FAULT: isize = -258;
pub const INCOMPATIBLE_THREADED_WORD: isize = -259;

/// Description of the exception
pub fn description(e: isize) -> &'static str {
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
        UNSUPPORTED_BASE_FOR_INTEGER_CONVERSION => "Unsupported base for integer conversion",
        INVALID_EXECUTION_TOKEN => "Invalid execution token",
        INTEGER_UNIDENTIFIED_FAULT => "Integer unidentified fault",
        INCOMPATIBLE_THREADED_WORD => "Incompatible threaded word",
        _ => "Unknown exception",
    }
}
