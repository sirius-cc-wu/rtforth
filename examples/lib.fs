-work

: evaluate
    begin parse-word
    token-empty? not  error? not  and
    while
    compiling? if compile-token ?stacks else interpret-token ?stacks then
    repeat ;
: quit
    reset
    begin accept evaluate
    ."  ok" flush
    again ;
: (abort) handle-error flush quit ;
' (abort) handler!

marker -work
