#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Exception {
    /// = -1, ANS Forth
    Abort = -1,
    /// = -2, ANS Forth
    AbortQuote = -2,
    /// = -3, ANS Forth
    StackOverflow = -3,
    /// = -4, ANS Forth
    StackUnderflow = -4,
    /// = -5, ANS Forth
    ReturnStackOverflow = -5,
    /// = -6, ANS Forth
    ReturnStackUnderflow = -6,
    /// = -7, ANS Forth
    DoLoopNestedTooDeeply = -7,
    /// = -8, ANS Forth
    DictionaryOverflow = -8,
    /// = -9, ANS Forth
    InvalidMemoryAddress = -9,
    /// = -10, ANS Forth
    DivisionByZero = -10,
    /// = -11, ANS Forth
    ResultOutOfRange = -11,
    /// = -12, ANS Forth
    ArgumentTypeMismatch = -12,
    /// = -13, ANS Forth
    UndefinedWord = -13,
    /// = -14, ANS Forth
    InterpretingACompileOnlyWord = -14,
    /// = -15, ANS Forth
    InvalidForget = -15,
    /// = -16, ANS Forth
    AttemptToUseZeroLengthString = -16,
    /// = -17, ANS Forth
    PicturedNumericOutputStringOverflow = -17,
    /// = -18, ANS Forth
    ParsedStringOverflow = -18,
    /// = -19, ANS Forth
    DefinitionNameTooLong = -19,
    /// = -20, ANS Forth
    WriteToAReadOnlyLocation = -20,
    /// = -21, ANS Forth
    UnsupportedOperation = -21,
    /// = -22, ANS Forth
    ControlStructureMismatch = -22,
    /// = -23, ANS Forth
    AddressAlignmentException = -23,
    /// = -24, ANS Forth
    InvalidNumericArgument = -24,
    /// = -25, ANS Forth
    ReturnStackImbalance = -25,
    /// = -26, ANS Forth
    LoopParametersUnavailable = -26,
    /// = -27, ANS Forth
    InvalidRecursion = -27,
    /// = -28, ANS Forth
    UserInterrupt = -28,
    /// = -29, ANS Forth
    CompilerNesting = -29,
    /// = -30, ANS Forth
    ObsolescentFeature = -30,
    /// = -31, ANS Forth
    ToBodyUsedOnNonCreatedDefinition = -31,
    /// = -32, ANS Forth
    InvalidNameArgument = -32,
    /// = -33, ANS Forth
    BlockReadException = -33,
    /// = -34, ANS Forth
    BlockWriteException = -34,
    /// = -35, ANS Forth
    InvalidBlockNumber = -35,
    /// = -36, ANS Forth
    InvalidFilePosition = -36,
    /// = -37, ANS Forth
    FileIOException = -37,
    /// = -38, ANS Forth
    NonExistentFile = -38,
    /// = -39, ANS Forth
    UnexpectedEndOfFile = -39,
    /// = -40, ANS Forth
    InvalidBaseForFloatingPointConversion = -40,
    /// = -41, ANS Forth
    LossOfPrecision = -41,
    /// = -42, ANS Forth
    FloatingPointDividedByZero = -42,
    /// = -43, ANS Forth
    FloatingPointResultOutOfRange = -43,
    /// = -44, ANS Forth
    FloatingPointStackOverflow = -44,
    /// = -45, ANS Forth
    FloatingPointStackUnderflow = -45,
    /// = -46, ANS Forth
    FloatingPointInvalidArgument = -46,
    /// = -47, ANS Forth
    CompilationWordListDeleted = -47,
    /// = -48, ANS Forth
    InvalidPostpone = -48,
    /// = -49, ANS Forth
    SearchOrderOverflow = -49,
    /// = -50, ANS Forth
    SearchOrderUnderflow = -50,
    /// = -51, ANS Forth
    CompilationWordListChanged = -51,
    /// = -52, ANS Forth
    ControlFlowStackOverflow = -52,
    /// = -53, ANS Forth
    ExceptionStackOverflow = -53,
    /// = -54, ANS Forth
    FloatingPointUnderflow = -54,
    /// = -55, ANS Forth
    FloatingPointUnidentifiedFault = -55,
    /// = -56, ANS Forth
    Quit = -56,
    /// = -57, ANS Forth
    ExceptionInSendingOrReceivingACharacter = -57,
    /// = -58, ANS Forth
    BracketIfElseOrThenException = -58,
    /// Request to run rtForth's inner loop, used in rtForth's core
    /// only. rtForth
    Nest = -9997,
    /// Request to exit rtForth's inner loop temporarily. rtForth
    Pause = -9998,
    /// request to leave the application. rtForth
    Bye = -9999,
}

impl Exception {
    /// Description of the exception
    pub fn description(&self) -> &'static str {
        match *self {
            Exception::Abort => "Aborted",
            Exception::AbortQuote => "Aborted",
            Exception::StackOverflow => "Stack overflow",
            Exception::StackUnderflow => "Stack underflow",
            Exception::ReturnStackOverflow => "Return stack overflow",
            Exception::ReturnStackUnderflow => "Return stack underflow",
            Exception::DoLoopNestedTooDeeply => "Do-loop nested too deeply",
            Exception::DictionaryOverflow => "Dictionary overflow",
            Exception::InvalidMemoryAddress => "Invalid memory address",
            Exception::DivisionByZero => "Division by zero",
            Exception::ResultOutOfRange => "Result out of range",
            Exception::ArgumentTypeMismatch => "Argument type mismatch",
            Exception::UndefinedWord => "Undefined word",
            Exception::InterpretingACompileOnlyWord => "Interpreting a compile only word",
            Exception::InvalidForget => "Invalid FORGET",
            Exception::AttemptToUseZeroLengthString => "Attempt to use zero length string",
            Exception::PicturedNumericOutputStringOverflow => {
                "Picture numeric output string overflow"
            }
            Exception::ParsedStringOverflow => "Parsed string overflow",
            Exception::DefinitionNameTooLong => "Definition name too long",
            Exception::WriteToAReadOnlyLocation => "Write to a read only location",
            Exception::UnsupportedOperation => "Unsupported operation",
            Exception::ControlStructureMismatch => "Control structure mismatch",
            Exception::AddressAlignmentException => "Address alignment exception",
            Exception::InvalidNumericArgument => "Invalid numeric argument",
            Exception::ReturnStackImbalance => "Return stack imbalance",
            Exception::LoopParametersUnavailable => "Loop parameters unavailable",
            Exception::InvalidRecursion => "Invalid recursion",
            Exception::UserInterrupt => "User interrupt",
            Exception::CompilerNesting => "Compiler nesting",
            Exception::ObsolescentFeature => "Obsolescent feature",
            Exception::ToBodyUsedOnNonCreatedDefinition => ">BODY used on non-CREATEd definition",
            Exception::InvalidNameArgument => "Invalid name argument",
            Exception::BlockReadException => "Block read exception",
            Exception::BlockWriteException => "Block write exception",
            Exception::InvalidBlockNumber => "Invalid block number",
            Exception::InvalidFilePosition => "Invalid file position",
            Exception::FileIOException => "File I/O exception",
            Exception::NonExistentFile => "Non-existent file",
            Exception::UnexpectedEndOfFile => "Unexpected end of file",
            Exception::InvalidBaseForFloatingPointConversion => {
                "Invalid BASE for floating point conversion"
            }
            Exception::LossOfPrecision => "Loss of precision",
            Exception::FloatingPointDividedByZero => "Floating point divided by zero",
            Exception::FloatingPointResultOutOfRange => "Floating point result out of range",
            Exception::FloatingPointStackOverflow => "Floating point stack overflow",
            Exception::FloatingPointStackUnderflow => "Floating point stack underflow",
            Exception::FloatingPointInvalidArgument => "Floating point invalid argument",
            Exception::CompilationWordListDeleted => "Compilation word list deleted",
            Exception::InvalidPostpone => "Invalid POSTPONE",
            Exception::SearchOrderOverflow => "Search order overflow",
            Exception::SearchOrderUnderflow => "Search order underflow",
            Exception::CompilationWordListChanged => "Compilation word list changed",
            Exception::ControlFlowStackOverflow => "Control flow stack overflow",
            Exception::ExceptionStackOverflow => "Exception stack overflow",
            Exception::FloatingPointUnderflow => "Floating point underflow",
            Exception::FloatingPointUnidentifiedFault => "Floating point unidentified fault",
            Exception::Quit => "QUIT",
            Exception::ExceptionInSendingOrReceivingACharacter => {
                "Exception in sending or receiving a character"
            }
            Exception::BracketIfElseOrThenException => "[IF],[ELSE],[THEN] exception",
            Exception::Nest => "Nest",
            Exception::Pause => "Pause",
            Exception::Bye => "Bye",
        }
    }
}
