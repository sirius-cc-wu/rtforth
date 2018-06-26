# 判斷與迴圈

: max ;
: min ;

## 亂數

[Xorshift 亂數產生器](https://en.wikipedia.org/wiki/Xorshift)
```
: rnd ( n -- x ) dup 13 lshift xor dup 17 rshift xor dup 5 lshift xor ;
```

## Guessing game

```
\ Start the guessing game, n is a non-zero number which is a seed to generate
\ the hidden number x on stack. Number x should be positive and less than 100. 
: start ( n -- x ) ; 
\ IF n > x, say "Tool high." to user. If n < x, say "Tool low.". If n = x, say "Correct.".
: guess ( x n -- ) ;
```
### 迴圈

: emits ;
: spaces ;
: ?dup ;

本書建議儘量使用 `begin ... while ... repeat` 而不使用 `begin ... until`，因為使用後者常犯所謂差一的錯誤。

例子：

-------------------------------------
## 本章指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `if` | ( -- ) &emsp;  | if |
| `else` | ( -- ) &emsp;  | else |
| `case` | ( -- ) &emsp;  | case |
| `endcase` | ( -- ) &emsp;  | endcase |
| `of` | ( -- ) &emsp;  | of |
| `endof` | ( -- ) &emsp;  | endof |
| `then` | ( -- ) &emsp;  | then |
| `begin` | ( -- ) &emsp;  | begin |
| `while` | ( -- ) &emsp;  | while |
| `repeat` | ( -- ) &emsp;  | repeat |
| `until` | ( -- ) &emsp;  | until |
| `do` | ( -- ) &emsp;  | do |
| `loop` | ( -- ) &emsp;  | loop |
| `+loop` | ( -- ) &emsp;  | plus-loop |