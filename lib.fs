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
: 2@ ( a-addr -- x1 x2 ) dup cell+ @ swap @ ;
: 2! ( x1 x2 a-addr -- ) swap over !  cell+ ! ;
: +! ( n|u a-addr -- ) dup @ rot + swap ! ;
: max ( n1 n2 -- n3 ) 2dup < if nip else drop then ;
: min ( n1 n2 -- n3 ) 2dup < if drop else nip then ;
: c, ( char -- ) here 1 chars allot c! ;

marker empty
