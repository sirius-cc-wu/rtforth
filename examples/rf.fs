-work

: evaluate
    begin parse-word
      token-empty? not  error 0=  and
    while
      compiling? if compile-token ?stacks else interpret-token ?stacks then
    repeat ;
: quit
    reset
    begin accept evaluate
    ."  ok" flush
    again ;
: (abort)
    0stacks error if
      .token space .error space 0error
    then flush quit ;
' (abort) handler!

marker -work
