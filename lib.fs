32 constant bl
: space ( -- )   32 emit ;
: spaces ( n -- )   0 begin 2dup > while 1+ space repeat 2drop ;
: . ( n -- )   0 .r space ;
: f. ( F: r -- )   0 7 f.r space ;
: decimal   10 base ! ;
: hex   16 base ! ;
: h. ( n1 -- )   base @ swap  hex . base ! ;
: <= ( n1 n2 -- flag)   > invert ;
: >= ( n1 n2 -- flag)   < invert ;
: f> ( -- flag ) ( F: r1 r2 -- )  f< invert ;
: ?dup ( x -- 0 | x x )   dup if dup then ;
: cr ( -- )   10 emit ;
: aligned ( addr -- a-addr )   1 cells 1- +  1 cells 1- invert and ;
: align ( -- )   here aligned  here - allot ;
: 2@ ( a-addr -- x1 x2 )   dup cell+ @ swap @ ;
: 2! ( x1 x2 a-addr -- )   swap over !  cell+ ! ;
: 2variable   align  create  2 cells allot ;
: +! ( n|u a-addr -- )   dup @ rot + swap ! ;
: max ( n1 n2 -- n3 )   2dup < if nip else drop then ;
: min ( n1 n2 -- n3 )   2dup < if drop else nip then ;
: c, ( char -- )   here 1 chars allot c! ;
\ Note: DO LOOP is not supported because their implementation is not compatible
\ to Forth 2012 standard.
\ : fill ( c-addr u char -- )
\     swap dup 0> if >r swap r>  0 do 2dup i + c! loop
\     else drop then 2drop ;
variable #tib  0 #tib !
variable tib 256 allot
: source ( -- c-addr u )   tib #tib @ ;
variable >in  0 >in !

marker -work
