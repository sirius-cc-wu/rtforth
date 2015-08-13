\ FILE:         ssieve-a.frt
\ LANGUAGE : ANSI Forth
\ COPYRIGHT :  Albert van der Horst FIG Chapter Holland
\ This program may be copied and distributed freely as is.
\ This program may only be modified
\ 1. with purpose :
\     configuration  or
\     to comment out parts of the program where ANSI words
\      are not available or non-ANSI words are already available  or
\     to improve the algorithm  or
\     to improve the documentation  or
\     to put the code in blocks  or
\     to port it to older (non ANS) systems
\ and
\ 2. Clear indication of changes and the person/company responsible for it.
\ Expressly forbidden is
\ 1. gratitious de-ansification, like use of non-standard comment symbols
\ 2. lower casing
\ 3. comment stripping
\ 4. addition of system-specific words like source control tools such as
\     MARKER look-alikes
\
\ PROJECT : NONE
\ CATEGORY : EXAMPLE
\ AUTHOR :  Albert van der Horst albert@spenarnc.xs4all.nl avdhorst@doge.nl
\ DESCRIPTION: Generates tables of prime numbers
\ VERSION : 3.2
\ CREATED : in the dark age of computing (1984) as an example Forth
\           program to be distributed on cassette tape
\ LAST CHANGE : 960425 AH ABORT's tested
\ LAST CHANGE : 960423 AH Further cleanup, ABORT on too large a range
\ LAST CHANGE : 960220 AH Further cleanup, make saving possible on chforth
\ LAST CHANGE : 951126 AH Cleanup to make it ANSI
\ LAST CHANGE : 911003 AH 2DROP added to LIST-PRIMES, cleanup's
\ 910730 Made totally silent
\ 910523 With help of L.Benschop, removed bug 10 million
\ 910521 ELIMINATE factored further, use of CLEAR-BIT
\ 910429 ELIMINATE factored out, before modification
\ 910406 Still BUG BY TESTING 10 MILLION

\ Super sieve, up to at least 10^9 by Albert van der HORST, HCC Forth gg

\ The well known sieve of Eratosthenes implemented in FORTH.
\ This implementation will spit out all primes up till at least the
\ precision of the square of MAXINT, so ca. 10^18 on a 32-bit machine. It
\ is done in batches, i.e. it is possible to calculate any range. Also it
\ has some beautiful printer arrangements to print your own books with
\ prime tables.

\ Some theoretisation.
\ This problem abounds with structure clashes. The number of primes that
\ fit on a page or line, the number {THOUSAND} of primes that in the user view
\ belong together, and the SIEVE-SIZE of a batch, and the number of primes that
\ fit in a byte, all clash.

\ some changes by anton

\ prelude of non-ANS words (anton)

: c+! ( c addr -- )
    dup c@ rot + swap c! ;

\ Comment out if not needed/present
    MARKER ssieve

\ PART 1 : TOOLS THAT MAY NOT BE PRESENT EVERYWHERE.

\ Convert unsigned U to double D
: U>D 0 ;        ( u U -- d D )

\ Increment the double at address A
: DINC  ( addr A -- .. )
    DUP >R 2@ 1. D+ R> 2!
;

\ Return the remainder REM of UD by U
\ UMD/MOD fails as soon as the quotient is more than fits in one cell
\ The following works over the full range.
: MMOD  ( ud UD, u U -- u REM )
    >R                     \ Save 16 bits number
    U>D R@ UM/MOD DROP     \ Kill multiples in m.s. cell
    R> UM/MOD DROP         \ Can't fail anymore
;


\ Create an area intended to contain a string consisting of
\ count, content and possibly a delimiter.
: STRING        ( "NAME" -- )
    CREATE 257 ALLOT
;

\ Fetch a constant string CS from S
\ This also shows the filosofy of these strings.
: $@     ( s S-- cs CS )
  COUNT ;

\ Store a constant string CS into S
: $!    ( cs CS, s S-- )
  2DUP C! 1+ SWAP CMOVE ;

\ Append a constant string CS to S
: $+! ( cs CS, s S-- )
   DUP C@ >R     \ remember old count
   2DUP C+!
   1+ R> + SWAP CMOVE
;

\ Append a character C to S
: APPEND-CHAR ( char C, s S-- )
  1 OVER C+!
  DUP C@ + C!
;


\ Remove leading spaces from a counted string S1 resulting in S2
: -LEADING      ( cs S1 -- cs S2 )
     BEGIN OVER C@ BL = OVER 0<> AND WHILE 1- SWAP 1+ SWAP REPEAT
;


\ PART 2: USER SPECIFIED CONFIGURATION

\ The user may change all of these
VARIABLE CH/L         \  Characters per line, not counting margins.
VARIABLE LN/P         \  Lines per page, not counting margins.
VARIABLE MORE         \  Flag: pause between pages
VARIABLE TOP-MARGIN   \  Lines before header starts
VARIABLE LEFT-MARGIN  \  Where the "thousands" are printed
VARIABLE ?SILENT      \  Flag: no output except for a first line

\ Set the I/O parameters above in a way that looks good on a terminal
: DEFAULT-IO    ( -- )
    80 CH/L !
    23 LN/P !       \ One line used for MORE message
    TRUE MORE !
    1 TOP-MARGIN !
    7 LEFT-MARGIN !
    FALSE ?SILENT !
;


\ PART 3: CALCULATION CONFIGURATION

\ It is dangerous if the prime table PRIMES should contain a number > MAX-U
\ so PRIMES-SIZE times 30 must not be larger than MAX-U
\ On the other hand the table is there for ?PRIME that
\ errs on the safe side if a number is not in the table.
 2184 CONSTANT PRIMES-SIZE    \ Prime table size.
\ WARNING: SIEVE-SIZE must not be negative (unsigned and > MAX-N)
\           Disasters will happen in ELIMINATE/30
30000 CONSTANT SIEVE-SIZE     \ As large as possible and larger than PRIMES-SIZE
    3 CONSTANT N-DIGITS \ Number of digits of mantissa
 1000 CONSTANT THOUSAND \ Must agree with N-DIGITS


\ PART 4: DATA STRUCTURES AND ELEMENTARY MANIPULATIONS

\  Both of the arrays contain bit tables, where each byte
\  represents a range i..i+29 where i is a multiple of 30.
\  In each 30 consequitive numbers precisely 8 have no divider
\  in common with 30.
\  Only these are the numbers that could be prime;
\  obviously numbers like 30*i+5 and 30*i+21 are no candidates.
\  SIEVE is reused, when more than "SIEVE-SIZE"*30 numbers are to be
\  investigated.
\  In that case we still need the single precision primes
\  to elimate numbers. Therefore the table PRIMES is always kept.
\  SIEVE is always used as a whole. Not all bits may be inspected

CREATE PRIMES PRIMES-SIZE ALLOT       \ Holds all single precision primes
CREATE SIEVE  SIEVE-SIZE ALLOT        \ Batch of candidates primes
SIEVE SIEVE-SIZE + CONSTANT END-SIEVE \ Where to stop sieving
VARIABLE PRIMES-FILLED    \ "The table prime is already filled"
FALSE PRIMES-FILLED !     \ Not yet.

\ Give the meaning of bits in a sieve table e.g. bit 7 of a byte
\ represents a 30-fold +29
CREATE BIT-VALUE
\   0     1     2     3     4     5     6     7
    1 C,  7 C, 11 C, 13 C, 17 C, 19 C, 23 C, 29 C,

( ************************)  HEX ( ************************)

\  The array S-MASK is such that for the number 30*i+j the j-th mask
\  gives the bit {in byte i} of that number, but 0 for non-candidates.
\  Used for Clearing bits.
CREATE S-MASK
\   0     1     2     3     4     5     6     7     8     9
   00 C, 01 C, 00 C, 00 C, 00 C, 00 C, 00 C, 02 C, 00 C, 00 C,
\  10    11    12    13    14    15    16    17    18    19
   00 C, 04 C, 00 C, 08 C, 00 C, 00 C, 00 C, 10 C, 00 C, 20 C,
\  20    21    22    23    24    25    26    27    28    29
   00 C, 00 C, 00 C, 40 C, 00 C, 00 C, 00 C, 00 C, 00 C, 80 C,

\ Compile the inverted mask of U.
: I,    ( u U -- )
    INVERT C,
;

\ Like C-MASK but inverted. Used for Setting bits.
CREATE C-MASK
\   0     1     2     3     4     5     6     7     8     9
   00 I, 01 I, 00 I, 00 I, 00 I, 00 I, 00 I, 02 I, 00 I, 00 I,
\  10    11    12    13    14    15    16    17    18    19
   00 I, 04 I, 00 I, 08 I, 00 I, 00 I, 00 I, 10 I, 00 I, 20 I,
\  20    21    22    23    24    25    26    27    28    29
   00 I, 00 I, 00 I, 40 I, 00 I, 00 I, 00 I, 00 I, 00 I, 80 I,

\ Preset SIEVE to all primes.
: INIT-T   SIEVE SIEVE-SIZE 0FF FILL ;

( ************************)  DECIMAL ( ************************)


\  In the following if the double number happens to be a
\  2,3 or 5 fold, one of the dummy masks is selected and
\  nothing changes; for TEST-B a FALSE is returned

\ Clears the bit of the prime candidate with (number) offset D in SIEVE
: CLEAR-B ( d D --  )
     30 UM/MOD                   \ Leave mask number and index
     SIEVE +                    \ Address in table
     SWAP C-MASK + C@           \ Get mask
     OVER C@ AND SWAP C!        \ Now clear that bit
;

\ Sets the bit of the prime candidate with (number) offset D in SIEVE
: SET-B ( d --  sets the bit of the prime candidate )
     30 UM/MOD                   \ Leave mask number and index
     SIEVE +                    \ Address in table
     SWAP S-MASK + C@           \ Get mask
     OVER C@ OR SWAP C!         \ Now clear that bit
;

\ Returns the bit of the prime candidate with (number) offset D in SIEVE
\ as FLAG.
\ WARNING: the result can be used by IF , but has not all bits set if true
: TEST-B ( d D -- n FLAG )
     30 UM/MOD                  \ Leave mask number and index
     SIEVE + C@                 \ Byte from table
     SWAP S-MASK + C@           \ Get mask
     AND                        \ Result: =0 or #0
;

\ Returns whether U is prime according to our small table PRIMES as FLAG.
\ It errs on the safe side, by saying yes if not present in table.
\ WARNING: the result can be used by IF , but has not all bits set if true
: PRIME? ( u U --  n FLAG )
     U>D 30 UM/MOD               \ Leave mask number and index
     DUP PRIMES-SIZE < 0= IF 2DROP TRUE EXIT THEN
     PRIMES + C@                \ Byte from table
     SWAP S-MASK + C@           \ Get mask
     AND                        \ Result: =0 or #0
;

\ Clears the bits that are zero in MASK at address A
: CLEAR-BIT ( u MASK,addr A --  )
     SWAP OVER C@               \ Old value
     AND SWAP C!                \ Now clear that bit
;

VARIABLE PRIME
\ Eliminate 1/30 of the multiples of prime in the SIEVE
\ i.e. all the multiples at BYTE-INDEX +x*PRIME
\ If the PRIME is large a wrap over all memory again into the table
\ might occurs.
\ If both the PRIME and the length of the table SIEVE
\ are smaller than MAX-U, this cannot occur.
: ELIMINATE/30       ( u MASK, n BYTE-INDEX -- )
   \ Skip if divisible by 2,3,5
   OVER [ HEX ] 0FF [ DECIMAL ] <> IF
       SIEVE +              \ Change to address in table SIEVE
       BEGIN
          DUP SIEVE END-SIEVE WITHIN
       WHILE
          2DUP CLEAR-BIT
          PRIME @ +          \  next addres to be cleared with the same MASK
       REPEAT
   THEN
   2DROP ( address & mask )
;

\ Calculate from the number index N-INDEX in table SIEVE two offsets: the
\ byte in the table where that number is represented, and the bit offset
\ within the table of masks
\ It is to be seen whether this is a good factorization
: SPLIT-INDEX   ( d N-INDEX -- n BYTE-INDEX, u BIT-INDEX )
      30 UM/MOD
;

\ The double at the low end of the SIEVE plus START is divisible by PRIME.
\ This is sufficient knowledge to eliminate all PRIME-fold's
\ from the table. For each possible mask there is a scan
\ through the table, for START, START+PRIME, START+2*PRIME etc.
\ NOTE : It is tricky, but PRIME and START count numbers in the table,
\ not bytes, not bits. That's why START is changed into a DOUBLE.
\ This once was a pretty bug. { Thanks Lennart! }
\ For the limitation of ELIMINATE/30 PRIME must be < MAX-N
: ELIMINATE ( n PRIME, u START -- )
    SWAP PRIME !
    U>D
    30 0 DO
      2DUP
      SPLIT-INDEX               \ Get index in SIEVE for start, and bit index
      SWAP C-MASK + C@  SWAP    \ Turn bit index into mask
      ELIMINATE/30              \ Eliminate all with the same mask
      PRIME @ U>D D+            \ The next starting point
    LOOP
    2DROP ( START)
;

\ As ELIMINATE however PRIME > MAX-N
\ We know that there is just one PRIME-fold present in the table
\ for each time around the loop.
: ELIMINATE-SINGLE ( u PRIME, u START -- )
    SWAP PRIME !
    U>D
    30 0 DO
        2DUP                    ( START[I] -- START[I], START[I] )
        SPLIT-INDEX             ( START[I] -- bit-index, byte-index )
        SWAP C-MASK + C@  SWAP  ( bit-index, byte-index -- mask, byte-index )
        SIEVE +                 ( byte-index -- sieve-address )
        DUP SIEVE END-SIEVE WITHIN IF CLEAR-BIT ELSE 2DROP THEN
                                ( mask, sieve-address -- )
        PRIME @ U>D D+          ( START[I] -- START[I+1] )
    LOOP
    2DROP ( START)
;


\ PART 5: INPUT OUTPUT

VARIABLE C#           \  character counter
VARIABLE L#           \  line counter
VARIABLE MANTISSA     \  The current thousands is to be printed
STRING RANGE-LOW$     \  The low end of the range
STRING RANGE-CURRENT$ \  The mantissa to be printed
VARIABLE TH-OFFSET    \  Digits after the mantissa corresponding to
                      \  start of current byte tested
STRING RANGE-HIGH$    \  The high end of the range

\ Outputs a range on screen. Remember that the RANGE$ has NDIGITS
\ zero's removed. Lateron maybe the numbers are adorned with comma's.
: .RANGE        ( s RANGE -- )
    $@ -LEADING
    DUP 0= IF
        2DROP 0 .
    ELSE
        TYPE
        N-DIGITS 0 DO [CHAR] 0 EMIT LOOP
     THEN
;

\ Form feed: start on new page
\ Even if ?SILENT still printed once to reflect input ????
: FORMFEED     ( -- )
      MORE @ ?SILENT @ 0= AND IF
         CR ." KEY FOR NEXT SCREEN"
        KEY [CHAR] Q = IF ABORT THEN
      THEN
      PAGE
      TOP-MARGIN @ 0 ?DO CR LOOP
      ." ERATOSTHENES SIEVE -- PRIMES BETWEEN "
      RANGE-LOW$ .RANGE ."  AND " RANGE-HIGH$ .RANGE CR
      TOP-MARGIN @ 1+ L# !
      TRUE MANTISSA !
;

\ Reserves L lines, by incrementing the line counter, give ff if needed
: ?P    ( n L -- )
    DUP L# @ +   LN/P @   > IF FORMFEED THEN
    L# +!
 ;

\ Start at a new line, maybe with a mantissa according to MANTISSA
: LINEFEED  ( -- )
      ?SILENT @ IF EXIT THEN
      1 ?P CR ( Checks first)
      MANTISSA @ IF
         RANGE-CURRENT$ $@ TYPE
      ELSE
         LEFT-MARGIN @ SPACES
      THEN
      LEFT-MARGIN @ C# !
      FALSE MANTISSA !
;

\ Reserves d chars, by incrementing the char counter, give lf if needed
: ?L    ( d L -- )
    DUP C# @ +   CH/L @   > IF LINEFEED THEN
    C# +!
;

\ Counts the number of primes
2VARIABLE COUNTER

\ COUNTER can be checked against the following table
\    X              PI(X) : Number of primes <= X :
\         1,000                168      Confirmed from litterature
\        10,000              1,229        "
\       100,000              9,592        "
\       900,000             71,274        "
\     1,000,000             78,498        "
\    10,000,000            664,579        "
\   100,000,000          5,761,455        "
\ 1,000,000,000         50,847,534        "
\ 2G145 .. 2G146            46,754      By this program
\ 2G .. 2G1 .            4,864,551        "
\ 4,000,000,000        189,961,812        "


\  Increment a number NUMBER, that consists of blanks followed by digits
: $INCREMENT    ( s NUMBER -- )
    $@ OVER + 1- SWAP 1- SWAP   ( s -- c-addr FIRST c-addr LAST )
    DO
        I C@ BL = IF            \ Replace blank to be incremented by '0'
            [CHAR] 0 I C!
        THEN
        1 I C+!
        I C@ [CHAR] 9 > IF
            -10 I C+!           \ Stay in loop to increment next digit
        ELSE
            LEAVE
        THEN
    -1 +LOOP
;

\ Makes sure that DECIMALS is less than THOUSAND
\ Adjust all offsets and start at a new line if larger
\ If all has been sieved, this word QUITs!
: ?TH-OFFSET            ( n DECIMALS -- n DECIMALS-MOD-THOUSAND  )
    THOUSAND /MOD IF
        THOUSAND NEGATE TH-OFFSET +!
        RANGE-CURRENT$ $INCREMENT
        RANGE-CURRENT$ $@ RANGE-HIGH$ $@ COMPARE 0= IF ABORT THEN
        1 MANTISSA !    LINEFEED
    THEN
;


\ Print the prime number, in fact only last NDIGIT decimals ``DECIMALS''
\ If DECIMALS has more digits, than ?TH-OFFSET takes care of that.
\ If negative, the input is ignored, to prevent printing things outside
\ of a range wanted.
: .PR   ( n DECIMALS -- )
  DUP 0< IF DROP EXIT THEN
  ?TH-OFFSET
  COUNTER DINC
  ?SILENT @ IF DROP EXIT THEN
  N-DIGITS 1+ ?L SPACE 0 <# N-DIGITS 0 DO # LOOP #> TYPE
;


\ Analyse BYTE according to the present coding.
\ Print the primes it represents. Assumes current offsets.
: PRINT-BYTE    ( char BYTE -- )
    0 SWAP              ( int BIT-VALUE-INDEX, char BYTE )
    BEGIN
        DUP WHILE
        DUP 1 AND IF    \ Test next bit of BYTE
            OVER BIT-VALUE + C@   TH-OFFSET @   + .PR
        THEN
        1 RSHIFT SWAP   \ Get next bit of BYTE
        1+ SWAP         \ Increment INDEX into BIT-VALUE
    REPEAT 2DROP
    30 TH-OFFSET +!     \ 30 numbers in this byte
;

\ If this module is used, this initialisation is obligatory

\ Force an initial formfeed and linefeed.
: INIT-P  CH/L @ C# !    LN/P @ L# ! ;


\ PART 6: SIEVING, at last

\ Return the amount of numbers that go in one batch
: BATCH-SIZE SIEVE-SIZE 30 UM* ; ( -- d)

\ Get a word from the input stream and put it at RANGE.
\ Pad at the front with blanks to have LEFT-MARGIN characters.
: GET-RANGE     ( addr RANGE -- )
     0 OVER C!                  \ Clear string
     BL WORD $@                 \ RANGE, addr, len
     ROT OVER                   \ addr, len, RANGE, len
     LEFT-MARGIN @ N-DIGITS +   \ addr, len, RANGE, len, margin
     2DUP > ABORT" RANGE HAS TOO MANY DIGITS"
     SWAP - 0 ?DO BL OVER APPEND-CHAR LOOP    \ addr, len, RANGE
     $+!
;

: FIXUP-RANGE          ( c-addr RANGE -- )
     N-DIGITS NEGATE SWAP C+!      \ Remove last three digits
;

\ Get a double number from a range string.
: RANGE>NUMBER          ( addr RANGE -- d )
     0. ROT $@ -LEADING
     DUP 0= IF DROP DROP EXIT THEN        \ Handle empty string
     >NUMBER ABORT" ILLEGAL CHARACTER IN LIMITS"
     DROP ( address left )
;

\ Return the number that corresponds to the current
\ byte investigated, apart from the offsets each bit in that
\ byte represents.
: CURRENT-NUMBER        ( -- ud CURRENT-NUMBER )
    RANGE-CURRENT$ RANGE>NUMBER   THOUSAND   1   M*/
    TH-OFFSET @ S>D   D+
;

\ Get the range limits from the input stream
: GET-LIMITS ( ud LIMIT -- . )
    RANGE-LOW$   DUP GET-RANGE RANGE>NUMBER
    RANGE-HIGH$  DUP GET-RANGE RANGE>NUMBER  D<
    0= ABORT" HIGH RANGE MUST BE HIGHER THAN LOW RANGE"
    RANGE-LOW$  FIXUP-RANGE
    RANGE-HIGH$ FIXUP-RANGE
    RANGE-LOW$ $@   RANGE-CURRENT$ $!           \ Start sieving at 0
    0 TH-OFFSET !                               \ Offset within small range
;

\ Sieve the first batch of SIEVE. Remember up to 30*"PRIMES-SIZE" primes in the
\ compact prime-table PRIMES.
: FILL-PRIMES
     PRIMES-FILLED @ 0= IF              \ We need to do this just once
         INIT-T
         1. CLEAR-B                     \ 1 is not prime.
         7
         BEGIN
              DUP S>D TEST-B IF
                DUP DUP DUP UM* D>S ELIMINATE        \ Pass prime, 2*prime as parameters
              THEN
              2 +
              DUP DUP UM*   PRIMES-SIZE 30 UM*   D< WHILE
         REPEAT DROP CR
         SIEVE PRIMES PRIMES-SIZE CMOVE            \ Save small primes
         TRUE PRIMES-FILLED !
     THEN
;

\ ROOT gives an approximation S for the square root of D to limit the
\ primes to investigate. It errs on the safe side, if D >MAX-D.
\ This is very inefficient for a 32 bit forth! ! !!!
HEX
: ROOT ( ud D -- u S   )
   2DUP 0. D< IF       \ Unsigned i.e. > MAX-D
     ABORT" UPPER BOUNDARY REACHED: CANNOT HANDLE LARGER NUMBERS"
  ELSE
    1 INVERT 1 DO
      I U>D D- I U>D D- 1. D-
      2DUP 0. D< IF
         2DROP I 1 + LEAVE
      THEN
    LOOP
  THEN
;
DECIMAL


\ Fill SIEVE.
\ Sieve a batch of primes, according to the current offset,
\ The manipulation with 1- handles the cases that the number at the start
\ of the batch is divisable by the prime number. We want an offset of 0
\ not one of prime.
: SIEVE-NEXT-BATCH      ( -- )
    INIT-T
    CURRENT-NUMBER                      \ Fix the lower number
    2DUP BATCH-SIZE D+ ROOT 7 DO        ( dlow )
        I PRIME? IF
            2DUP 1. D-   I   MMOD        ( dlow -- dlow, remainder-1 )
            I SWAP - 1-          ( remainder-1 -- First to be eliminated )
            I SWAP               ( First to be eliminated -- prime, ftbe )
            I 0> IF ELIMINATE ELSE ELIMINATE-SINGLE THEN
        THEN
    2 +LOOP
    2DROP
;


\ In the first batch it may be that the first number falls
\ in the middle of a byte in the PRIME table.
\ This is handled by TH-OFFSET being negative for the first byte.
\ All bits in the number resulting in a printout lower that CURRENT-NUMBER
\ are ignored by not printing negative numbers in .PR
: PRINT-FIRST-BATCH
    RANGE-LOW$ RANGE>NUMBER

    2DUP D0= IF
        2 .PR 3 .PR 5 .PR          \ These numbers are not represented in SIEVE
    THEN

    \ This comparison MUST be double precision
    2DUP THOUSAND 30 M*/   PRIMES-SIZE S>D D< IF
        2DUP THOUSAND 1 M*/
        30 UM/MOD               ( d Low -- skew, Index in PRIME )
        SWAP NEGATE TH-OFFSET !
        PRIMES +               \ Where to start in table
        PRIMES PRIMES-SIZE +   SWAP   DO
            I C@ PRINT-BYTE
        LOOP
    THEN
;

: PRINT-NEXT-BATCH
    SIEVE SIEVE-SIZE +   SIEVE   DO
        I C@ PRINT-BYTE
    LOOP
;

\ Make a list of primes between LOW and HIGH , from the input stream
\ Ranges are adjusted to be a multiple of THOUSAND
: (LIST-PRIMES)          ( "LOW" "HIGH" -- )
\ terminates by ABORTing
    GET-LIMITS
    0. COUNTER 2!
    INIT-P
    FILL-PRIMES
    PRINT-FIRST-BATCH
    BEGIN
        SIEVE-NEXT-BATCH
        PRINT-NEXT-BATCH
        TRUE WHILE              \ Broken by QUIT in .PR
    REPEAT
;

: LIST-PRIMES ( "LOW HIGH" -- )
    ['] (LIST-PRIMES) CATCH
    DUP -1 <> IF
	THROW
    THEN
    DROP ;

:  .HELP        ( -- )
        CR CR
        ." Eratosthenes super sieve version  3.2" CR
        ." Copyright Albert van der Horst, FIG chapter Holland" CR
        ." Type '.HELP' for this help" CR
        ." Type 'LIST-PRIMES <LOW> <HIGH>' for primes in that range " CR
        ?SILENT @ IF
           ." Only counting (?SILENT)" CR
        ELSE
           ." With output (?SILENT off)" CR
           CH/L ? ." characters per line (CH/L)" CR
           LN/P ? ." lines pro page (LN/P)" CR
           MORE @ IF ." Uses " ELSE ." No " THEN
           ." pausing between pages (MORE)" CR
        THEN
;

DEFAULT-IO  .HELP

: ??   counter 2@ D. ;
