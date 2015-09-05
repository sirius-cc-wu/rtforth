variable base
10 base !
: decimal 10 base ! ;
: ?dup ( x -- 0 | x x ) 0 <> if dup then ;
: cr ( -- ) 10 emit ;
32 constant bl
: space ( -- ) 32 emit ;
: spaces ( n -- ) 0 begin 2dup > while 1+ space repeat 2drop ;
: 2/ ( n -- n/2 ) 1 arshift ;
: 2* ( n -- n*2 ) 1 lshift ;
: aligned ( addr -- a-addr ) 1 cells 1- +  1 cells 1- invert and ;
: align ( -- ) here aligned  here - allot ;
marker empty
