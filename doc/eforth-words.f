( -*- forth -*- )

( this is adapted from Bill Muench's x86 eForth.. original copyright notice:

---
Copyright Bill Muench All rights reserved.
Permission is granted for non-commercial use, provided this notice is included.
Contact Bill Muench concerning commercial use.
---

contains some adaptations to make it work on the purrr kernel. i
converted everything to lower case. aestetics i guess..

)



( ============================================================ )

: noop ( -- ) ( 0x7b ) ;

( system variables )

: _var ( -- a ) ( 0xb9 ) r> ; compile-only
: _con ( -- n ) ( 0xba ) r> @ ; compile-only

create '?key ( input device vector )
  ' ?rx ,
create 'emit ( output device vector )
  ' tx! ,

variable base ( numeric radix ) ( 6.1.0750 )( 0xa0 )
variable dpl  ( numeric input decimal place )
variable hld  ( numeric output string pointer )

variable >in ( input buffer offset ) ( 6.1.0560 )
create #in   ( input buffer count )
  2 cells allot ( input buffer address )

variable csp ( save stack pointer )

create state ( interpret/compile flag ) ( 6.1.2250 )( 0xdc )
  2 cells allot ( interpret/compile vector )

create dp ( dictionary pointer )
  2 cells allot

create sup ( -- tid )
  =rp , ( return stack )
  =sp , ( data stack )

=bl constant bl ( -- c ) ( 6.1.0770 )( 0xa9 )

( common functions )

: hex ( -- ) ( 6.2.1660 ) 16 base ! ;
: decimal ( -- ) ( 6.1.1170 ) 10 base ! ;

: rot ( n1 n2 n3 -- n2 n3 n1 ) ( 6.1.2160 )( 0x4a ) >r swap r> swap ;
: nip ( n1 n2 -- n2 ) ( 6.2.1930 )( 0x4d ) swap drop ;
: 2drop ( n n -- ) ( 6.1.0370 )( 0x52 ) drop drop ;
: 2dup ( n1 n2 -- n1 n2 n1 n2 ) ( 6.1.0380 )( 0x53 ) over over ;
: ?dup ( n -- n n | 0 ) ( 6.1.0630 )( 0x50 ) dup if dup then ;

: + ( n n -- n ) ( 6.1.0120 )( 0x1e ) um+ drop ;
: d+ ( d d -- d ) ( 8.6.1.1040 )( 0xd8 ) >r  swap >r  um+  r> +  r> + ;

: invert ( n -- n ) ( 6.1.1720 )( 0x26 ) -1 xor ;

: negate ( n -- n ) ( 6.1.1910 )( 0x2c ) invert 1 + ;
: dnegate ( d -- d ) ( 8.6.1.1230 ) invert >r invert 1 um+ r> + ;

: s>d ( n -- d ) ( 6.1.2170 ) dup 0< ;
: abs ( n -- u ) ( 6.1.0690 )( 0x2d ) dup 0< if negate then ;
: dabs ( d -- ud ) ( 8.6.1.1160 ) dup 0< if dnegate then ;

: - ( n n -- n ) ( 6.1.0160 )( 0x1f ) negate + ;

: pick ( n -- n ) ( 6.2.2030 )( 0x50 )
  ?dup if swap >r 1 - recurse r> swap exit then dup ;

( comparison )

: 0= ( n -- f ) ( 6.1.0270 )( 0x34 ) if 0 exit then -1 ;
: = ( n n -- f ) ( 6.1.0530 )( 0x3c ) xor 0= ;

: u< ( u u -- f ) ( 6.1.2340 )( 0x40 ) 2dup xor 0< if  nip 0< exit then - 0< ;
:  < ( n n -- f ) ( 6.1.0480 )( 0x3a ) 2dup xor 0< if drop 0< exit then - 0< ;

: max ( n n -- n ) ( 6.1.1870 )( 0x2f ) 2dup      < if swap then drop ;
: min ( n n -- n ) ( 6.1.1880 )( 0x2e ) 2dup swap < if swap then drop ;

: within ( u ul uh -- f ) ( 6.2.2440 )( 0x45 ) over - >r - r> u< ;

( multiply )

: lshift ( u n -- u ) ( 6.1.1805 )( 0x27 )
  begin dup
  while >r  dup +  r> 1 -
  repeat drop ;

: um* ( u u -- ud ) ( 6.1.2360 )( 0xd4 )
  0 swap  [ #bits ] literal
  begin dup
  while >r  dup um+ >r >r  dup um+ r> + r>
    if >r over um+ r> + then  r> 1 -
  repeat drop >r  nip r> ;

: * ( n n -- n ) ( 6.1.0090 )( 0x20 ) um* drop ;

( divide )

: rshift ( u n -- u ) ( 6.1.2162 )( 0x28 )
  0 swap  [ #bits ] literal swap -
  begin dup
  while >r  2dup d+  r> 1 -
  repeat drop  nip ;

: um/mod ( ud u -- ur uq ) ( 6.1.2370 )( 0xd5 )
  2dup u<
  if negate  [ #bits ] literal
    begin dup
    while >r  >r  dup um+ >r >r  dup um+ r> +
      dup r> r@ swap >r um+  r> or
      if >r drop 1 + r> else drop then r>  r> 1 -
    repeat 2drop swap exit
  then drop 2drop  -1 dup ;

: sm/rem ( d n -- r q ) ( 6.1.2214 ) ( symmetric )
  over >r >r  dabs r@ abs um/mod
  r> r@ xor 0< if negate then  r> 0< if >r negate r> then ;

: fm/mod ( d n -- r q ) ( 6.1.1561 ) ( floored )
  dup 0<  dup >r if negate >r dnegate r> then
  >r dup 0< if r@ + then r> um/mod r> if >r negate r> then ;

: /mod ( n n -- r q ) ( 6.1.0240 )( 0x2a ) over 0< swap  fm/mod ; ( or sm/rem )

: mod ( n n -- r ) ( 6.1.1890 )( 0x22 ) /mod drop ;
: / ( n n -- q ) ( 6.1.0230 )( 0x21 ) /mod nip ;

( memory access )

: +! ( n a -- ) ( 6.1.0130 )( 0x6c ) dup >r @ + r> ! ;
: count ( a -- a c ) ( 6.1.0980 )( 0x84 ) dup char+ swap c@ ;
: bounds ( a n -- a+n a ) ( 0xac ) over + swap ;
: /string ( a u n -- a+n u-n ) ( 17.6.1.0245 ) dup >r - swap r> chars + swap ;
: aligned ( a -- a ) ( 6.1.0706 )( 0xae ) ( depends on 2's comp and 2^n cell si
ze )
  [ 1 cells 1 - dup ] literal + [ invert ] literal and ;

: 2! ( u u a -- ) ( 6.1.0310 )( 0x77 ) swap over ! cell+ ! ;
: 2@ ( a -- u u ) ( 6.1.0350 )( 0x76 ) dup cell+ @ swap @ ;

: move ( a a u -- ) ( 6.1.1900 )( 0x78 )
  >r  2dup u<
  if
    begin r> dup
    while char- >r  over r@ + c@  over r@ + c!
    repeat drop  2drop exit
  then r> over + >r
  begin dup r@ xor
  while >r  dup c@ r@ c!  char+ r> char+
  repeat r> drop  2drop ;

: fill ( a u c -- ) ( 6.1.1540 )( 0x79 )
  >r  chars bounds
  begin 2dup xor
  while r@ over c!  char+
  repeat r> drop 2drop ;

: -trailing ( a u -- a u ) ( 17.6.1.0170 )
  begin dup
  while 1 -  2dup chars + c@  bl swap u<
  until 1 + then ;

: >adr ( xt -- a ) ; \ itc
: >body ( xt -- a ) ( 6.1.0550 )( 0x86 ) >adr cell+ cell+ ; \ itc

( multitask )

variable up ( current task pointer )
: _usr ( -- a ) up @ r> @ + ; compile-only

( u1\tf\tid\tos\status\follower\r>--<s  order is important )
1 cells ( init offset )
  cell- dup user follower ( address of next task's status )
  cell- dup user status   ( pass or wake )
  cell- dup user tos      ( top of stack )
  cell- dup user tid      ( back link tid )
  cell- dup user tf       ( throw frame )
  cell- dup user u1       ( free )
drop ( cleanup )

: 's ( tid a -- a ) ( index another task's local variable )
  follower  cell+ - swap @ + ;

: _pass ( -- ) ( hilevel absolute branch )
  r> @ >r ; compile-only
' _pass constant pass

: _wake ( -- ) ( restore follower )
  r> up !  tos @ sp! rp! ; compile-only
' _wake constant wake

: pause ( -- ) ( allow another task to execute )
  rp@  sp@ tos !  follower @ >r ;

: stop ( -- ) ( sleep current task )
  pass status ! pause ; compile-only

: get ( semaphore -- )
  pause ( remember your manners )
  dup @ status xor ( owner ? )
  if begin dup @ while pause repeat ( no, wait for release )
    status swap ! ( lock ) exit
  then drop ;

: release ( semaphore -- )
  dup @ status xor if drop exit then  0 swap ! ( unlock ) ;

: sleep ( tid -- ) ( sleep another task )
  pass swap status 's ! ;

: awake ( tid -- ) ( wake another task )
  wake swap status 's ! ;

: activate ( tid -- )
  dup 2@        ( tid sp rp )
  r> over !     ( save entry at rp )
  over !        ( save rp at sp )
  over tos 's ! ( save sp in tos )
  awake ; compile-only

: build ( tid -- )
  dup sleep                     ( sleep new task )
  follower @ over follower 's ! ( link new task )
  dup status 's follower !      ( link old task )
  dup tid 's ! ;                ( link to tid )

( numeric input )

: digit? ( c base -- u f ) ( 0xa3 )
  >r [char] 0 - 9 over <
  if 7 - dup 10 < or then dup r> u< ;

: >number ( ud a u -- ud a u ) ( 6.1.0570 )
  begin dup
  while >r  dup >r c@ base @ digit?
  while swap base @ um* drop rot base @ um* d+ r> char+ r> 1 -
  repeat drop r> r> then ;

: number? ( a u -- d -1 | a u 0 )
  over c@ [char] - = dup >r if 1 /string then
  >r >r  0 dup  r> r>  -1 dpl !
  begin >number dup
  while over c@ [char] . xor
    if rot drop rot r> 2drop  0 exit
    then 1 - dpl !  char+  dpl @
  repeat 2drop r> if dnegate then -1 ;

( numeric output )

: here ( -- a ) ( 6.1.1650 )( 0xad ) dp @ ;
: pad ( -- a ) ( 6.2.2000 ) here [ #pad chars ] literal + ;

: <# ( -- ) ( 6.1.0490 )( 0x96 ) pad hld ! ;
: digit ( u -- c ) 9 over < 7 and + [char] 0 + ;
: hold ( c -- ) ( 6.1.1670 )( 0x95 ) hld @ char- dup hld ! c! ;

: # ( d -- d ) ( 6.1.0030 )( 0xc7 )
  0 base @ um/mod >r base @ um/mod swap digit hold r> ;

: #s ( d -- d ) ( 6.1.0050 )( 0xc8 ) begin # 2dup or 0= until ;
: #> ( d -- a u ) ( 6.1.0040 )( 0xc9 ) 2drop hld @ pad over - ;

: sign ( n -- ) ( 6.1.2210 )( 0x98 ) 0< if [char] - hold then ;

( error handling )

: catch ( xt -- 0 | err ) ( 9.6.1.0875 )( 0x217 )
  sp@ >r  tf @ >r  rp@ tf !  execute  r> tf !  r> drop  0 ;

: throw ( 0 | err -- | err ) ( r: i*x i*y -- i*x i*y | i*x ) ( 9.6.1.2275 )( 0x
218 )
  ?dup if tf @ rp!  r> tf !  r> swap >r sp! drop r> then ;

: abort ( i*n -- ) ( r: i*x i*y -- i*x ) ( 9.6.2.0670 )( 0x216 ) -1 throw ;

( basic i/o )

: ?key ( -- c -1 | 0 )  pause  '?key @ execute ;
: key ( -- c ) ( 6.1.1750 )( 0x8e ) begin ?key until ;
: nuf? ( -- f ) ?key dup if 2drop key [ =cr ] literal = then ;

: emit ( c -- ) ( 6.1.1320 )( 0x8f ) 'emit @ execute ;
: space ( -- ) ( 6.1.2220 ) bl emit ; ,c" coyote"

: emits ( n c -- )
  swap 0 max begin dup while over emit 1 - repeat 2drop ;
: spaces ( n -- ) ( 6.1.2230 ) bl emits ;

: type ( a u -- ) ( 6.1.2310 )( 0x90 )
  chars bounds begin 2dup xor while count emit repeat 2drop ;
: cr ( -- ) ( 6.1.0990 )( 0x92 ) [ =cr ] literal emit [ =lf ] literal emit ;

: _" ( -- a )
  r> r> dup count chars + aligned >r swap >r ; compile-only

: _s" ( -- a u ) _" count ; compile-only
: _." ( -- ) ( 0x12 ) _" count type ; compile-only
: _abort" ( i*n f -- i*n | ) ( r: i*x i*y -- i*x i*y | i*x )
  if _" csp ! -2 throw then _" drop ; compile-only

: s.r ( a u n -- ) over - spaces type ;
: d.r ( d n -- ) ( 8.6.1.1070 ) >r dup >r dabs <# #s r> sign #> r> s.r ;
: u.r ( u n -- ) ( 6.2.2330 )( 0x9c ) 0 swap d.r ;
: .r ( n n -- ) ( 6.2.0210 )( 0x9e ) >r s>d r> d.r ;

: d. ( d -- ) ( 8.6.1.1060 ) 0 d.r space ;
: u. ( u -- ) ( 6.1.2320 )( 0x9b ) 0 d. ;
: . ( n -- ) ( 6.1.0180 )( 0x9d ) base @ 10 xor if u. exit then s>d d. ;
: ? ( a -- ) ( 15.6.1.0600 ) @ . ;

( bits & bytes )

: pack ( a1 u a2 -- a2 ) ( 0x83 )
  over 256 u<
  if dup >r  over >r  char+ swap chars move  r> r@ c!  r> exit
  then -18 throw ;

: depth ( -- n ) ( 6.1.1200 )( 0x51 )
  sp@  tid @ cell+ @  swap - [ 1 cells ] literal / ;
: ?stack ( -- ) depth 0< abort" depth?" ;

( terminal )

: accept ( a u -- u ) ( 6.1.0695 )
  over + over ( bot eot cur )
  begin key
    dup [ =cr ] literal xor ( carrage return ? )
  while
    dup [ =bs ] literal = ( backspace ? )
    if ( destructive backspace )
      drop  >r over r@ < dup ( any chars ? )
      if [ =bs ] literal dup emit  bl emit  emit
      then r> +
    else ( printable )
      >r  2dup xor ( more ? )
      if r@ over c!  char+  r@ emit
      then r> drop
    then
  repeat drop  nip  swap - ;

( interpreter )

: same? ( a a u -- f ) \ ???faster chars
  swap >r
  begin dup
  while char-  2dup + c@  over r@ + c@  xor
  until r> drop 2drop  0 exit ( no match )
  then r> drop 2drop  -1 ; ( found )

: _delimit ( a u -- a u delta ) \ ???chars
  bounds  dup >r  char-
  begin char+  2dup xor ( skip leading bl )
  while bl over c@ <
  until swap over ( save first non blank addr )
    begin char+  2dup xor ( scan trailing bl )
    while dup c@  bl 1 +  <
    until nip  dup char+ ( found )
    else drop dup ( not found )
    then >r  over -  r>
  else drop 0 over ( all bl )
  then r> - ;

: _parse ( a1 u1 c -- a1 u2 delta ) \ ???chars
  >r  over +  over char- ( save char, adjust addr )
  begin char+  2dup xor ( inc addr ? )
  while dup c@ r@ = ( match ? )
  until swap r> 2drop  over -  dup 1 + exit ( found )
  then  swap r> 2drop  over -  dup ; ( not found )

: name> ( a -- xt ) count chars + char+ aligned ;

: wid? ( a u wid -- xt lex -1 | a u 0 ) \ ???chars
  swap >r  @ ( address of last word )
  begin dup ( last word ? )
  while count r@ = ( count ? )
    if 2dup r@ same? ( match )
      if swap r> 2drop char-
        dup name>  swap count chars + c@  -1 exit ( found )
      then
    then char-  cell- @ ( link )
  repeat drop r>  0 ; ( no match )

create context ( search order )
  #vocs 1 + cells allot ( wids )

: sfind ( a u -- xt lex -1 | a u 0 )
  context cell- >r ( setup )
  begin r> cell+ dup >r @ dup ( wid | 0 )
  while wid? ( found ? )
  until -1 then r> drop ;

: _[ ( a u -- ) ( the forth interpreter )
  sfind ( search dictionary )
  if [ =comp ] literal and abort" compile?"
    execute ?stack exit
  then
  number? ( unknown symbol, try to convert a number )
  if dpl @ 0< ( single? )
    if drop then exit
  then -13 throw ; compile-only
: [ ( -- ) ( 6.1.2500 ) ['] _[  0  state 2! ; immediate

: source ( -- a u ) ( 6.1.2216 ) #in 2@ ;
: parse-word ( "ccc" -- a u ) source >in @ /string _delimit >in +! ;

: evaluate ( a u -- ) ( 6.1.1360 )( 0xcd )
  >in @ >r  0 >in !  source >r >r  #in 2!
  begin parse-word dup
  while state cell+ @ execute
  repeat 2drop  r> r> #in 2!  r> >in ! ;

( redirect input ms-dos only =============================== )

: asciiz ( a u a -- a )
  dup >r  swap chars  2dup + 0 swap c!  move  r> ;

: stdin ( a u -- )
  here asciiz redirect abort" file?" ; compile-only

: from ( "ccc" -- ) ( chain not nest )
  parse-word stdin  source >in ! drop ;

( ========================================================== )

create 'ok ( prompt options )
  ' noop , ( typically .s )
: quit ( -- ) ( r: i*x -- ) ( 6.1.2050 )
  sup @ rp!         ( reset return stack )
  [ ' [ compile, ]  ( reset interpret state )
  s" con" stdin     ( reset console i/o, ms-dos only )
  begin
    begin
      [ =tib ] literal  ( input buffer )
      dup [ #tib ] literal accept space ( user input )
      ['] evaluate catch dup ( error ? )
      if dup -1 xor      ( abort  = -1 )
        if cr dup -2 xor ( abort" = -2 )
          if source drop ( undefined error )
            >in @ -trailing type ."  ?(" 0 .r ." )"
          else csp @ count type
          then space
        then
        sup cell+ @ sp! ( reset data stack )
        recurse         ( restart )
      then cr state @ = ( 0 from catch )
    until 'ok @ execute ." ok " ( prompt )
  again ;

( compiler )

: align ( -- ) ( 6.1.0705 ) here aligned dp ! ;

: allot ( n -- ) ( 6.1.0710 ) dp +! ;
: s, ( a u -- ) here  over chars char+ allot  pack drop ;
: c, ( n -- ) ( 6.1.0860 )( 0xd0 ) here  [ 1 chars ] literal allot  c! ;
: , ( n -- ) ( 6.1.0150 )( 0xd3 ) here  [ 1 cells ] literal allot  ! ;

: compile, ( xt -- ) ( 6.2.0945 )( 0xdd ) , ;
: literal ( n -- ) ( 6.1.1780 ) ['] _lit compile, , ; immediate

: char ( "ccc" -- c ) ( 6.1.0895 ) parse-word drop c@ ;
: [char] ( "ccc" -- ) ( 6.1.2520 ) char  [ ' literal compile, ] ; immediate

: ' ( "name" -- xt ) ( 6.1.0070 ) parse-word sfind if drop exit then -13 thr
ow ;
: ['] ( "name" -- ) ( 6.1.2510 ) '  [ ' literal compile, ] ; immediate

: parse ( c "ccc" -- a u ) ( 6.2.2008 ) \ ???move
  >r source >in @ /string r> _parse >in +! ;

: ( ( "comment" -- ) ( 6.2.0200 ) [char] ) parse type ; immediate
: ( ( "comment" -- ) ( 6.1.0080 ) [char] ) parse 2drop ; immediate
: \ ( "comment" -- ) ( 6.2.2535 ) source >in ! drop ; immediate

: sliteral ( a u -- ) ( -- a u ) ( 17.6.1.2212 )
  ['] _s" compile, s, align ; immediate compile-only

: ,c" ( "ccc" -- ) [char] " parse s, align ;

: s" ( "ccc" -- ) ( 6.1.2165 ) ['] _s" compile, ,c" ; immediate compile-only
: ." ( "ccc" -- ) ( 6.1.0190 ) ['] _." compile, ,c" ; immediate compile-only

: abort" ( "ccc" -- ) ( 6.1.0680 )
  ['] _abort" compile, ,c" ; immediate compile-only

: _] ( a u -- ) ( the forth compiler )
  sfind ( search dictionary )
  if [ =imed ] literal and
    if execute ?stack exit ( immediate )
    then compile, exit
  then
  number? ( unknown symbol, try to convert a number )
  if dpl @ 0<
    if drop ( single )
    else swap  [ ' literal compile, ] ( double )
    then  [ ' literal compile, ] exit
  then -13 throw ; compile-only
: ] ( -- ) ( 6.1.2540 ) align ['] _] -1 state 2! ;

create forth-wordlist ( -- wid ) ( 16.6.1.1595 )
  0 , ( na, of last definition, linked )
  0 , ( wid|0, next or last wordlist in chain )
  0 , ( na, wordlist name pointer )

create last ( -- a )
  1 cells allot ( na, of last definition, unlinked )
  1 cells allot ( wid, current wordlist for linking )
label =token
  1 cells allot ( xt, of last definition )

create current ( -- a )
  forth-wordlist , ( wid, new definitions )
  forth-wordlist , ( wid, head of chain )

: get-current ( -- wid ) ( 16.6.1.1643 ) current @ ;
: set-current ( wid -- ) ( 16.6.1.2195 ) current ! ;
: definitions ( -- ) ( 16.6.1.1180 ) context @ set-current ;

: ?unique ( a u -- a u )
  2dup  get-current wid?
  if 2drop cr ." redef " 2dup type exit then 2drop ;

: head, ( "name" -- ) \ ???fix ( xt "name" -- )
  parse-word  dup
  if ?unique ( warn if redefined )
    align
    get-current  dup @ ,  here last 2! ( link )
    dup c, ( save count )
    here swap  dup allot  move ( build name )
    0 c, ( build attribute byte )
    exit
  then -16 throw ; ( attempt to use zero-length string )

| : lex! ( u -- ) last @ count chars + dup >r c@ or r> c! ;
: immediate ( -- ) ( 6.1.1710 ) [ =imed ] literal  lex! ;
: compile-only ( -- ) [ =comp ] literal  lex! ;

: reveal ( -- ) last 2@ swap ! [ ' [ compile, ] ;
: recurse ( -- ) ( 6.1.2120 ) [ =token ] literal @ compile, ; immediate

: postpone ( "name" -- ) ( 6.1.2033 )
  parse-word sfind
  if [ =imed ] literal and if compile, exit then
    [ ' literal compile, ]  ['] compile, compile,  exit
  then -13 throw ; immediate

( defining words )

: code ( "name" -- ) ( 15.6.2.0930 ) \ itc
  head, align here cell+ , reveal ;

: next, ( -- ) \ itc 80x86 only
  [ next1 ] literal  h# e9 c,  here 2 + - , ;

: :noname ( -- xt ) ( 6.2.0455 ) \ itc
  align here  dup [ =token ] literal !  [ list1 ] literal , ] ;

: : ( "name" -- ) ( 6.1.0450 ) head, :noname drop ;
: ; ( -- ) ( 6.1.0460 ) ['] exit compile, reveal ; immediate compile-only

: _does> ( -- ) ( link child )
\  align ( child ) \ ???why
  r>  [ =token ] literal @  cell+ ( itc )  ! ; compile-only

: does> ( -- ) ( 6.1.1250 ) ( build parent )
  ['] _does> compile, ( link child )
  :noname drop  ['] r> compile, ( begin child )
; immediate compile-only

: create ( "name" -- ) ( 6.1.1000 ) ['] _var  : reveal compile, ;
: variable ( "name" -- ) ( 6.1.2410 ) create 0 , ;
: constant ( n "name" -- ) ( 6.1.0950 ) ['] _con  : reveal compile,  , ;

: user ( n "name" -- ) ['] _usr  : reveal compile,  , ;

: hat ( u s r "name" -- ) ( -- tid )
  create + swap [ 7 cells ] literal + ( tf\tid\tos\status\follower\r>--<s )
  dup here + ( rp0 ) , + dup here + ( sp0 ) , allot ;

: wordlist ( -- wid ) ( 16.6.1.2460 )
  align here 0 ,  dup current cell+  dup @ ,  !  0 , ;

: order@ ( a -- u*wid u )
  dup @ dup if >r cell+  recurse  r> swap 1 + exit then nip ;
: get-order ( -- u*wid u ) ( 16.6.1.1647 ) context order@ ;

: set-order ( u*wid n -- ) ( 16.6.1.2197 )
  dup -1 = if drop forth-wordlist 1 then ( default ? )
  [ #vocs ] literal over u< if -46 throw then ( range ? )
  context swap
  begin dup
  while >r  swap over !  cell+  r> 1 -
  repeat  ( 0 ) swap ! ;

\ ============================================================
: _marker ( -- ) ( r: dfa -- ) \ ???
  r> 2@ ( * ) dup @ follower !  dup context
  begin >r cell+ dup @ dup r@ ! while r> cell+ repeat ( search order )
  cell+ dup 2@ current 2!  cell+ dup @ ( cur wid & head )
  begin >r  cell+ dup @ r@ !  r> cell+ @ ?dup 0= until ( wid last na's )
  r> 2drop ( * ) dp 2! ; compile-only

: marker ( "name" -- ) \ ???
  align dp 2@ ( * ) follower @ ,  context
  begin dup @ dup , while cell+ repeat  drop ( search order )
  current 2@ , dup , ( cur wid & head )
  begin dup @ , cell+ @ ?dup 0= until ( wid last na's )
  ['] _marker : reveal compile, ( * ) , , ;
\ ============================================================

( control flow )

: begin ( -- a ) ( 6.1.0760 ) here ; immediate
: then ( a -- ) ( 6.1.2270 ) [ ' begin compile, ] ( over - ) swap ! ; immediate

: resolve ( a -- ) ( [ ' begin compile, ] - ) , ;
: mark ( -- a ) here [ ' begin compile, ] resolve ;

: if ( -- a ) ( 6.1.1700 ) ['] _if compile, mark ; immediate
: ahead ( -- a ) ( 15.6.2.0702 ) ['] _else compile, mark ; immediate
: else ( a -- a ) ( 6.1.1310 ) [ ' ahead compile, ] swap [ ' then compile, ] ;
immediate
: while ( a -- a a ) ( 6.1.2430 ) [ ' if compile, ] swap ; immediate

: until ( a -- ) ( 6.1.2390 ) ['] _if compile, resolve ; immediate
: again ( a -- ) ( 6.2.0700 ) ['] _else compile, resolve ; immediate
: repeat ( a a -- ) ( 6.1.2140 ) [ ' again compile, ' then compile, ] ; immedia
te

( tools )

: .s ( -- ) ( 15.6.1.0220 )( 0x9f )
  ?stack depth begin ?dup while dup pick . 1 - repeat ;

: !csp ( -- ) sp@ csp ! ;
: ?csp ( -- ) sp@ csp @ xor abort" csp?" ;

: >char ( c -- c )
  h# 7f and dup 127 bl within if drop [char] _ then ;

: _type ( a u -- ) ( alpha dump )
  chars bounds begin 2dup xor while count >char emit repeat 2drop ;

: _dump ( a u -- ) ( numeric dump )
  chars bounds begin 2dup xor while count 3 u.r repeat 2drop ;

: dump ( a u -- ) ( 15.6.1.1280 )
  base @ >r hex  chars bounds
  begin 2dup swap u< while ( range? )
    cr dup 0 <#  # # # #  #> type ( address )
    space [ #dump ] literal  2dup _dump ( numeric )
    space space  2dup _type ( alpha )
    chars +  nuf? ( user? )
  until then 2drop  r> base ! ;

: .id ( a -- ) count _type ;

: widwords ( a u wid -- a u )
  swap >r  dup
  if cr dup ." wid=" u. cr
    begin @ dup ( last name ? )
    while 2dup char+ r@ same? ( match ? )
      if dup .id space then cell-  nuf?
    until then
  then drop r> ;
: words ( "ccc" -- )
  bl parse  dup
  if current begin cell+ @ ?dup while dup >r widwords r> repeat ( all wid )
  else context @ widwords
  then 2drop ;

: named? ( aa -- na | 0 )
  current ( all wid )
  begin cell+ @ dup ( last link ? )
  while dup >r
    begin @ ?dup ( zero link ? )
    while 2dup name> >adr = ( match ? )
       if swap r> 2drop exit ( found )
       then cell-
    repeat r>
  repeat nip ( not found ) ;

: ssee ( a u -- ) ( simple decompiler )
  cells bounds
  begin 2dup xor ( done? )
  while dup named? ?dup if cr .id cr then
    space dup @ >adr named? ?dup
    if .id ( display named token )
    else dup @ 0 u.r ( unnamed token )
    then cell+  nuf?
  until then 2drop ;
: see ( "name" -- ) ( 15.6.1.2194 ) ' >adr -1 ssee ;

( software reset )

: cold ( -- )
  sup 2@ rp! sp! ( init stacks )
  sup @ cell- ( follower ) up ! ( init user pointer )
  status follower !  sup tid !  sup awake ( init tasks )
  0 !io ( init i/o device )
  hex  -1 set-order definitions
  cr [ =version ] literal count type
  cr [ =(c) ] literal count type
  cr quit ;

code bye ( -- ) ( 15.6.2.0830 )
  h# 20 int ( terminate process )
end-code

proc vcold             ( cold start entry )
  cli                  ( disable interrupt for old 808x cpu bug )
  cs ax mov  ax ds mov ( ds=cs )
  ax ss mov            ( ss=cs )
  sup ## bp mov        ( system user pointer )
  1 cells bp [] sp mov ( init sp )
  0 cells bp [] bp mov ( init rp )
  sti                  ( enable interrrupts )
\ =====================
  reset ## dx mov      ( ^c on output ms-dos only )
  h# 2523 ## ax mov    ( set ^c interrupt int23 )
  h# 21 int
\ =====================
  cld                  ( direction flag, increment )
  ' cold ## di mov     ( first word to execute ) \ itc
  0 di [] jmp          ( start eforth )
end-code

cr ( metacompile end ) ]meta

references

   1. mailto:<forth@(www.)calcentral.com>bill muench?subject=eforth license