# Forth 2012 Compatibility

The following words are not compatible to ANS Forth:

* PARSE
* FLUSH

## 6.1 Core words

Section number | Definition name | Compatibility
---------------|-----------------|--------------
6.1.0010 | ! | Y
6.1.0030 | # |
6.1.0040 | #> |
6.1.0050 | #S |
6.1.0070 | ' | Y
6.1.0080 | ( | Y
6.1.0090 | * | Y
6.1.0100 | */ | N, do not support mixed-precision.
6.1.0110 | */MOD | N, do not support mixed-precision.
6.1.0120 | + | Y
6.1.0130 | +! | Y
6.1.0140 | +LOOP | Y
6.1.0150 | , | Y
6.1.0160 | - | Y
6.1.0180 | . | Y
6.1.0190 | ." | Y
6.1.0230 | / | Y
6.1.0240 | /MOD | Y
6.1.0250 | 0< | Y
6.1.0270 | 0= | Y
6.1.0290 | 1+ | Y
6.1.0300 | 1- | Y
6.1.0310 | 2! | Y
6.1.0320 | 2* |
6.1.0330 | 2/ |
6.1.0350 | 2@ | Y
6.1.0370 | 2DROP | Y
6.1.0380 | 2DUP | Y
6.1.0400 | 2OVER | Y
6.1.0430 | 2SWAP | Y
6.1.0450 | : | Y
6.1.0460 | ; | Y
6.1.0480 | < | Y
6.1.0490 | <# |
6.1.0530 | = | Y
6.1.0540 | > | Y
6.1.0550 | >BODY | Y
6.1.0560 | >IN |
6.1.0570 | >NUMBER |
6.1.0580 | >R | Y
6.1.0630 | ?DUP | Y
6.1.0650 | @ | Y
6.1.0670 | ABORT |
6.1.0680 | ABORT" |
6.1.0690 | ABS | Y
6.1.0695 | ACCEPT |
6.1.0705 | ALIGN | Y
6.1.0706 | ALIGNED | Y
6.1.0710 | ALLOT | Y
6.1.0720 | AND | Y
6.1.0750 | BASE | Y
6.1.0760 | BEGIN | Y
6.1.0770 | BL | Y
6.1.0850 | C! | Y
6.1.0860 | C, | Y
6.1.0870 | C@ | Y
6.1.0880 | CELL+ | Y
6.1.0890 | CELLS | Y
6.1.0895 | CHAR | Y
6.1.0897 | CHAR+ | Y
6.1.0898 | CHARS | Y
6.1.0950 | CONSTANT | Y
6.1.0980 | COUNT | Y
6.1.0990 | CR | Y
6.1.1000 | CREATE | Y
6.1.1170 | DECIMAL | Y
6.1.1200 | DEPTH | Y
6.1.1240 | DO | Y
6.1.1250 | DOES> | Y
6.1.1260 | DROP | Y
6.1.1290 | DUP | Y
6.1.1310 | ELSE | Y
6.1.1320 | EMIT | Y
6.1.1345 | ENVIRONMENT? |
6.1.1360 | EVALUATE |
6.1.1370 | EXECUTE | Y
6.1.1380 | EXIT | Y
6.1.1540 | FILL | Y
6.1.1550 | FIND |
6.1.1561 | FM/MOD | N, do not support mixed-precision
6.1.1650 | HERE | Y
6.1.1670 | HOLD |
6.1.1680 | I | Y
6.1.1700 | IF | Y
6.1.1710 | IMMEDIATE | Y
6.1.1720 | INVERT | Y
6.1.1730 | J | Y
6.1.1750 | KEY | N
6.1.1760 | LEAVE | Y
6.1.1780 | LITERAL | Y
6.1.1800 | LOOP | Y
6.1.1805 | LSHIFT | Y
6.1.1810 | M* | N, do not support mixed-precision
6.1.1870 | MAX | Y
6.1.1880 | MIN | Y
6.1.1890 | MOD | Y
6.1.1900 | MOVE | Y
6.1.1910 | NEGATE | Y
6.1.1980 | OR | Y
6.1.1990 | OVER | Y
6.1.2033 | POSTPONE | Y
6.1.2050 | QUIT |
6.1.2060 | R> | Y
6.1.2070 | R@ | Y
6.1.2120 | RECURSE |
6.1.2140 | REPEAT | Y
6.1.2160 | ROT | Y
6.1.2162 | RSHIFT | Y
6.1.2165 | S" | Y
6.1.2170 | S>D | N, do not support double-precision
6.1.2210 | SIGN |
6.1.2214 | SM/REM | N, do not support mixed-precision
6.1.2216 | SOURCE | TODO
6.1.2220 | SPACE | Y
6.1.2230 | SPACES | Y
6.1.2250 | STATE |
6.1.2260 | SWAP | Y
6.1.2270 | THEN | Y
6.1.2310 | TYPE | Y
6.1.2320 | U. | N, do not support unsigned integer
6.1.2340 | U< | N, do not support unsigned integer
6.1.2360 | UM* | N, do not support mixed-precision
6.1.2370 | UM/MOD | N
6.1.2380 | UNLOOP | Y
6.1.2390 | UNTIL | Y
6.1.2410 | VARIABLE | Y
6.1.2430 | WHILE | Y
6.1.2450 | WORD | Y
6.1.2490 | XOR | Y
6.1.2500 | [ | Y
6.1.2510 | ['] | Y
6.1.2520 | [CHAR] | Y
6.1.2540 | ] | Y

## 6.2 Core extension words

Section number | Definition name | Compatibility
---------------|-----------------|--------------
6.2.0200 | .( | Y
6.2.0210 | .R | Y
6.2.0260 | 0<> | Y
6.2.0280 | 0> | Y
6.2.0340 | 2>R | Y
6.2.0410 | 2R> | Y
6.2.0415 | 2R@ | Y
6.2.0455 | :NONAME |
6.2.0500 | <> | Y
6.2.0620 | ?DO | Y
6.2.0698 | ACTION-OF |
6.2.0700 | AGAIN | Y
6.2.0825 | BUFFER: |
6.2.0855 | C" |
6.2.0873 | CASE | Y
6.2.0945 | COMPILE, | Y
6.2.1173 | DEFER | Y
6.2.1175 | DEFER! | Y
6.2.1177 | DEFER@ | Y
6.2.1342 | ENDCASE | Y
6.2.1343 | ENDOF | Y
6.2.1350 | ERASE |
6.2.1485 | FALSE | Y
6.2.1660 | HEX | Y
6.2.1675 | HOLDS |
6.2.1725 | IS |
6.2.1850 | MARKER | Y
6.2.1930 | NIP | Y
6.2.1950 | OF | Y
6.2.2000 | PAD | Y
6.2.2008 | PARSE | TODO
6.2.2020 | PARSE-NAME |
6.2.2030 | PICK |
6.2.2125 | REFILL |
6.2.2148 | RESTORE-INPUT | TODO
6.2.2150 | ROLL |
6.2.2266 | S\" | Y
6.2.2182 | SAVE-INPUT | TODO
6.2.2218 | SOURCE-ID | TODO
6.2.2295 | TO |
6.2.2298 | TRUE | Y
6.2.2300 | TUCK | Y
6.2.2330 | U.R | N, do not support unsigned integer
6.2.2350 | U> | N, do not support unsigned integer
6.2.2395 | UNUSED |
6.2.2405 | VALUE |
6.2.2440 | WITHIN | Y
6.2.2530 | [COMPILE] |
6.2.2535 | \ | Y

## 8.6.1 Double-Number words

Section number | Definition name | Compatibility
---------------|-----------------|--------------
8.6.1.0360 2CONSTANT | Y
8.6.1.0390 2LITERAL | Y
8.6.1.0440 2VARIABLE | Y

Other words are not planned.

## 11.6.1 File Access words

Section number | Definition name | Compatibility
---------------|-----------------|--------------
11.6.1.0080 | ( | TODO
11.6.1.0765 | BIN | Y
11.6.1.0900 | CLOSE-FILE | Y
11.6.1.1010 | CREATE-FILE | Y
11.6.1.1190 | DELETE-FILE | Y
11.6.1.1520 | FILE-POSITION | Y
11.6.1.1522 | FILE-SIZE | Y
11.6.1.1717 | INCLUDE-FILE | TODO
11.6.1.1718 | INCLUDED | TODO
11.6.1.1970 | OPEN-FILE | Y
11.6.1.2054 | R/O | Y
11.6.1.2056 | R/W | Y
11.6.1.2080 | READ-FILE | Y
11.6.1.2090 | READ-LINE | N, because without buffer it's not efficient.
11.6.1.2142 | REPOSITION-FILE | Y
11.6.1.2147 | RESIZE-FILE | Y
11.6.1.2165 | S" | TODO
11.6.1.2218 | SOURCE-ID | TODO
11.6.1.2425 | W/O | Y
11.6.1.2480 | WRITE-FILE | Y
11.6.1.2485 | WRITE-LINE | N, because without buffer it's not efficient.

## 12.6.1 Floating-Point words

Section number | Definition name | Compatibility
---------------|-----------------|--------------
12.6.1.0558 | >FLOAT |
12.6.1.1130 | D>F | N, do not support double-precision
12.6.1.1400 | F! | Y
12.6.1.1410 | F* | Y
12.6.1.1420 | F+ | Y
12.6.1.1425 | F- | Y
12.6.1.1430 | F/ | Y
12.6.1.1440 | F0< | Y
12.6.1.1450 | F0= | Y
12.6.1.1460 | F< | Y
12.6.1.1470 | F>D | N, do not support double-precision
12.6.1.1472 | F@ | Y
12.6.1.1479 | FALIGN | Y
12.6.1.1483 | FALIGNED | Y
12.6.1.1492 | FCONSTANT | Y
12.6.1.1497 | FDEPTH |
12.6.1.1500 | FDROP | Y
12.6.1.1510 | FDUP | Y
12.6.1.1552 | FLITERAL | Y
12.6.1.1555 | FLOAT+ | Y
12.6.1.1556 | FLOATS | Y
12.6.1.1558 | FLOOR | Y
12.6.1.1562 | FMAX | Y
12.6.1.1565 | FMIN | Y
12.6.1.1567 | FNEGATE | Y
12.6.1.1600 | FOVER | Y
12.6.1.1610 | FROT | Y
12.6.1.1612 | FROUND | Y
12.6.1.1620 | FSWAP | Y
12.6.1.1630 | FVARIABLE | Y
12.6.1.2143 | REPRESENT |

## 12.6.2 Floating-Point extension words

Section number | Definition name | Compatibility
---------------|-----------------|--------------
12.6.2.1203 | DF! | N, do not support double precision
12.6.2.1204 | DF@ | N, do not support double precision
12.6.2.1205 | DFALIGN | N, do not support double precision
12.6.2.1207 | DFALIGNED | N, do not support double precision
12.6.2.1207.40 | DFFIELD: | N, do not support double precision
12.6.2.1208 | DFLOAT+ | N, do not support double precision
12.6.2.1209 | DFLOATS | N, do not support double precision
12.6.2.1415 | F** | Y
12.6.2.1427 | F. | Y
12.6.2.1471 | F>S | Y
12.6.2.1474 | FABS | Y
12.6.2.1476 | FACOS | Y
12.6.2.1477 | FACOSH |
12.6.2.1484 | FALOG |
12.6.2.1486 | FASIN | Y
12.6.2.1487 | FASINH |
12.6.2.1488 | FATAN | Y
12.6.2.1489 | FATAN2 | Y
12.6.2.1491 | FATANH |
12.6.2.1493 | FCOS | Y
12.6.2.1494 | FCOSH |
12.6.2.1513 | FE. |
12.6.2.1515 | FEXP |
12.6.2.1516 | FEXPM1 |
12.6.2.1517 | FFIELD: |
12.6.2.1553 | FLN |
12.6.2.1554 | FLNP1 |
12.6.2.1557 | FLOG |
12.6.2.1613 | FS. |
12.6.2.1614 | FSIN | Y
12.6.2.1616 | FSINCOS | Y
12.6.2.1617 | FSINH |
12.6.2.1618 | FSQRT | Y
12.6.2.1625 | FTAN | Y
12.6.2.1626 | FTANH |
12.6.2.1627 | FTRUNC |
12.6.2.1628 | FVALUE |
12.6.2.1640 | F~ | Y
12.6.2.2035 | PRECISION |
12.6.2.2175 | S>F | Y
12.6.2.2200 | SET-PRECISION |
12.6.2.2202 | SF! | N, do not support single float
12.6.2.2203 | SF@ | N, do not support single float
12.6.2.2204 | SFALIGN | N, do not support single float
12.6.2.2206 | SFALIGNED | N, do not support single float
12.6.2.2206.40 | SFFIELD: | N, do not support single float
12.6.2.2207 | SFLOAT+ | N, do not support single float
12.6.2.2208 | SFLOATS | N, do not support single float

## 15.6.1 Programming-Tools words

Section number | Definition name | Compatibility
---------------|-----------------|--------------
15.6.1.0220 | .S | Y
15.6.1.0600 | ? | Y
15.6.1.1280 | DUMP | Y
15.6.1.2194 | SEE |
15.6.1.2465 | WORDS | Y
