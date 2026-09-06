-work

: evaluate-input
    begin parse-word
      token-empty? not
    while
      compiling? if compile-token ?stacks else interpret-token ?stacks then
    repeat ;

: quit
    reset
    begin receive evaluate-input
    ."  ok" flush-output
    again ;

: (abort)
    0stacks error -2 1 within not if
      .token space .error
    then flush-output 0error quit ;

\ Cold start
: cold
    2 halt  3 halt  4 halt  5 halt
    ['] (abort) handler!  quit ;

marker -work
