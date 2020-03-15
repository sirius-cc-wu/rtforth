: 2* ( n -- n*2 )   2 * ;
: 2/ ( n -- n/2 )   2 / ;
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
: chars ( n -- n1 )  ; immediate
: c, ( char -- )   here 1 chars allot c! ;
: fill ( c-addr u char -- )
    swap dup 0> if >r swap r>  0 do 2dup i + c! loop
    else drop then 2drop ;
: count ( a -- a+1 n )  dup c@  swap 1 +  swap ;
: /string ( c-addr1 u1 n -- c-addr2 u2 ) ( 17.6.1.0245 )  dup >r - swap r> chars + swap ;
: append ( c-addr1 u c-addr2 - )  2>r  2r@ count + swap move  2r> dup >r c@ + r> c! ;
: variable   create  0 , ;
: on ( a -- )   true swap ! ;
: off ( a -- )   false swap ! ;
: literal ( n -- )   postpone lit  , ; immediate compile-only
: 2literal ( n1 n2 -- )
    swap postpone lit  ,  postpone lit  , ; immediate compile-only
: fliteral ( F: r -- )   postpone flit  f, ; immediate compile-only
: 2constant   create 2, does>  2@ ;
: 2variable   create  0 , 0 , ;
: fvariable   create falign 0e f, does> faligned ;
: +field ( n1 n2 -- n3 )   create over , + does> @ + ;
: defer   create ['] noop ,  does> @ execute ;
: defer@ ( xt1 -- xt2 )   >body @ ;
: defer! ( xt2 xt1 -- )   >body ! ;

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
: word ( char -- c-addr )   dup _skip  _parse  here !token  here ;

\ Execution time
: xtime ( xt -- )   utime >r >r r@ execute r> r> (xtime) ;

\ Search-order word set
0 constant forth-wordlist
1 constant optimizer-wordlist

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

\ File access
0 constant r/o
1 constant w/o
2 constant r/w
: bin ( -- )   ;

\ Input source
: _save-input ( -- source-id source-idx 2 )   source-id  source-idx 2 ;
: _restore-input ( source-id source-idx 2 -- )
    2 = if source-idx! source-id! else abort then ;
defer save-input   ' _save-input  ' save-input  defer!
defer restore-input   ' _restore-input  ' restore-input  defer!

\ Stack to save & restore source
\ content: | capacity | count=N | source-idx1 | source-id1 | ... | source_idxN | source-idN |
\ NOTE: multitasking is not considered here.
create src-stack 16 , 0 , 16 2* cells allot
: save-source
    source-id source-idx
    src-stack 2@ over > if
      ( src-id src-idx count ) 1+ dup src-stack cell+ !
      ( src-id src-idx count+1 ) 2* cells src-stack +  2!
    else abort then ;
: restore-source
    src-stack 2@ drop  dup 0 > if
      ( count ) dup 1- src-stack cell+ !
      ( count ) 2* cells src-stack + 2@
      source-idx!  source-id!
    else abort then ;
: evaluate-input
    begin parse-word
      token-empty? not
    while
      compiling? if compile-token
      ?stacks else interpret-token ?stacks then
    repeat ;
\ Multitasking is not considered here.
variable load-line#
: load-source-file ( -- )
    begin
      source-id load-line
    while
      drop
      0 source-idx!
      evaluate-input  flush-output
      1 load-line# +!
    repeat  drop ;
: included ( c-addr u -- )
    2dup  r/o open-file 0= if
        save-source
        ( c-addr u file-id ) open-source source-id!
        postpone [
        1 load-line# !
        load-source-file
        source-id  restore-source  close-source
    else
        abort
    then
;
: include ( "path" -- )   32 word count included ;
: \\ ( -- )   source-id   begin  dup load-line  while  drop  repeat  2drop ;
marker -work
