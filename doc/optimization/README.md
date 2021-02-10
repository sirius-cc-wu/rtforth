# Optimization

Different designs of inner interpreter.

* new-run: function call with dfa, stack and stack pointer as arguments.
* bytecode: switch threaded code.
* tco: use same stack frame as caller through tail/sibling call optimization. Still not available with run compiler.
* call: subroutine threaded call
* asm: use asm! to write a small subroutine threaded forth core..
