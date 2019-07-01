# Forth 2012 Compatibility

The following words are not compatible to ANS Forth:

* parse 
* flush

## Core words compatility

Section number | Definition name | Compatibility
---------------|-----------------|--------------
6.1.0010 | ! | Y
6.1.0030 | # |
6.1.0040 | #> |
6.1.0050 | #S |
6.1.0070 | ' | Y
6.1.0080 | ( | Y
6.1.0090 | * | Y
6.1.0100 | */ |
6.1.0110 | */MOD |
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
6.1.0550 | >BODY |
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
6.1.1561 | FM/MOD |
6.1.1650 | HERE | Y
6.1.1670 | HOLD |
6.1.1680 | I | Y
6.1.1700 | IF | Y
6.1.1710 | IMMEDIATE | Y
6.1.1720 | INVERT | Y
6.1.1730 | J | Y
6.1.1750 | KEY |
6.1.1760 | LEAVE | Y
6.1.1780 | LITERAL |
6.1.1800 | LOOP | Y
6.1.1805 | LSHIFT | Y
6.1.1810 | M* |
6.1.1870 | MAX | Y
6.1.1880 | MIN | Y
6.1.1890 | MOD | Y
6.1.1900 | MOVE |
6.1.1910 | NEGATE | Y
6.1.1980 | OR | Y
6.1.1990 | OVER | Y
6.1.2033 | POSTPONE |
6.1.2050 | QUIT |
6.1.2060 | R> | Y
6.1.2070 | R@ | Y
6.1.2120 | RECURSE |
6.1.2140 | REPEAT | Y
6.1.2160 | ROT | Y
6.1.2162 | RSHIFT | Y
6.1.2165 | S" |
6.1.2170 | S>D |
6.1.2210 | SIGN |
6.1.2214 | SM/REM |
6.1.2216 | SOURCE |
6.1.2220 | SPACE | Y
6.1.2230 | SPACES | Y
6.1.2250 | STATE |
6.1.2260 | SWAP | Y
6.1.2270 | THEN | Y
6.1.2310 | TYPE | Y
6.1.2320 | U. |
6.1.2340 | U< |
6.1.2360 | UM* |
6.1.2370 | UM/MOD |
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
