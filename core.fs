32 constant bl
: space ( -- )   32 emit ;
: spaces ( n -- )   0 begin 2dup > while 1+ space repeat 2drop ;
: . ( n -- )   0 .r space ;
: f. ( F: r -- )   0 7 f.r space ;
: ? ( addr -- )   @ . ;
: decimal   10 base ! ;
: hex   16 base ! ;
: h. ( n -- )   base @ swap  hex . base ! ;
: h.r ( n1 n2 -- )   base @ >r  hex .r  r> base ! ;
: <= ( n1 n2 -- flag)   > invert ;
: >= ( n1 n2 -- flag)   < invert ;
: f> ( -- flag ) ( F: r1 r2 -- )  fswap f< ;
: ?dup ( x -- 0 | x x )   dup if dup then ;
: tuck ( n1 n2 -- n2 n1 n2 )   swap over ;
: cr ( -- )   10 emit ;
: f, ( F: r -- )   here  1 floats allot  f! ;
: 2@ ( a-addr -- x1 x2 )   dup cell+ @ swap @ ;
: 2! ( x1 x2 a-addr -- )   swap over !  cell+ ! ;
: +! ( n|u a-addr -- )   dup @ rot + swap ! ;
: 2, ( n1 n2 -- )   here  2 cells allot  2! ;
: max ( n1 n2 -- n3 )   2dup < if nip else drop then ;
: min ( n1 n2 -- n3 )   2dup < if drop else nip then ;
: chars ( n -- n1 ) ( 6.1.0898 )  ; immediate
: c, ( char -- )   here 1 chars allot c! ;
: fill ( c-addr u char -- )
    swap dup 0> if >r swap r>  0 do 2dup i + c! loop
    else drop then 2drop ;
: count ( a -- a+1 n ) ( 6.1.0980 )  dup c@  swap 1 +  swap ;
: /string ( c-addr1 u1 n -- c-addr2 u2 ) ( 17.6.1.0245 )  dup >r - swap r> chars + swap ;
: append ( c-addr1 u c-addr2 - )  2>r  2r@ count + swap move  2r> dup >r c@ + r> c! ;
: variable   create  0 , ;
: does> ( -- )   postpone _does  postpone exit ; immediate compile-only
: literal ( n -- ) ( 6.1.1780 )   postpone lit  , ; immediate compile-only
: 2literal ( n1 n2 -- ) ( 8.6.1.0390 )
    swap postpone lit  ,  postpone lit  , ; immediate compile-only
: fliteral ( F: r -- ) ( 12.6.1.1552 )   postpone flit  f, ; immediate compile-only
: 2constant   create 2, does>  2@ ;
: 2variable   create  0 , 0 , ;
: fvariable   create falign 0e f, does> faligned ;
: +field ( n1 n2 -- n3 )   create over , + does> @ + ;
variable #tib  0 #tib !
variable tib 256 allot
: source ( -- c-addr u )   tib #tib @ ;
variable >in  0 >in !
: pad ( -- addr )   here 512 + aligned ;

\ Dump
: bounds ( a n -- a+n a )   over + swap ;
: >char ( c -- c )
  $7f and dup bl 127 within invert if drop [char] _ then ;
: _type ( a u -- )
  chars bounds begin 2dup xor while count >char emit repeat 2drop ;
: _dump ( a u -- )
    chars bounds begin 2dup xor while count 3 h.r repeat 2drop ;
: dump ( a u -- ) ( 15.6.1.1280 )
    chars bounds
    begin  2dup swap <  while
      dup 4 h.r  [char] : emit  ( address )
      space  8 2dup _dump
      space space  2dup _type
      chars +  cr
    repeat  2drop ;

\ WORD
: word ( char -- c-addr )   dup _skip  parse  here !token  here ;

\ Execution time
: xtime ( t0 xt -- )   2>r r@ execute 2r> (xtime) ;

\ Multitasker
0 constant operator
: nod   begin pause again ;
: halt ( n -- )   activate nod ;
: stop   me suspend pause ;
\ Aquire facility `a`.
: get ( a -- )   begin  dup @  while pause repeat me swap ! ;
\ Release facility `a`.
: release ( a -- )   dup @ me = if 0 swap ! else drop then ;
\ Wait `n` milli-seconds.
: ms ( n -- )   mtime  begin mtime over -  2 pick <  while pause repeat  2drop ;

marker -work
