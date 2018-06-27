# 選擇與重覆

## 選擇

在之前我們學過 `min` 這個指令。現在我們看看它是怎麼用冒號定義出來的：

```
: min ( n1 n2 -- n3 )   2dup < if drop else nip then ;
```
當 `n1<n2` 時選擇 n1，否則選擇 n2。

Forth 表達選擇的方式是 ( 條件 ) if A else B then C。條件是在堆疊上的一個真假值。

指令 `if` 會檢查堆疊上這個數值，如果為真，即使只是部份為真，只要不是 0，就執行 A ，否則執行 B ，最後都會執行 C。請參照以下的流程圖：

```
( 條件 ) if A else B then C

               |
               v
         +-----------+
         | 條件為真或  | No
         | 一部份為真  |--------+
         +-----------+        |
               | Yes          |
               v              v
          +---------+    +---------+
          |    A    |    |    B    |
          +---------+    +---------+
               |              |
               |<-------------+
               v
          +---------+
          |    C    |
          +---------+
               |
```
當沒有 B 這個分支時，可以只使用 `if` 和 `then`。比如以下這個 `?dup` 指令，當堆疊上的數字不為 0 時才會複制一份。

```
: ?dup ( n -- 0 | n n )  dup if dup then ;
```
在此我們在堆疊效果中使用垂直線 `|` 來表示堆疊的結果是 ( 0 ) 或 ( n n ) 中的一個。

練習：請以冒號定義一個行為和  `abs` 一樣的 `my-abs`。
```
rf> : my-abs ( n -- |n| )   dup 0< if negate then ;
 ok
rf> 3 my-abs .   -3 my-abs .
3 3  ok
rf> .s
 ok
```

練習：請以冒號定義一個行為和  `max` 一樣的 `my-max`。
```
rf> : my-max ( n1 n2 -- n1 | n2 )   2dup < if nip  else drop then ;
 ok
rf> 1 3 my-max .  3 1 my-max .
3 3  ok
rf> .s
 ok
```
在最後我們用 `.s` 檢查的確堆疊上沒留下不該留的資料。透過這種互動，可以很快的找出 Forth 程式中的 bug。而不需要依賴除錯器 (debugger)。

指令 `if` 、 `else` 、 `then` 只能用於冒號定義中。它們和 `."` 以及  `;` 一樣，都是只能用於冒號定義中的「編譯指令」。如果用在冒號定義之外，會出現錯誤訊息。
```
rf> if
Interpreting a compile only word
```

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `if` | ( -- ) &emsp;  | if |
| `else` | ( -- ) &emsp;  | else |
| `then` | ( -- ) &emsp;  | then |
| `?dup` | ( -- ) &emsp;  | question-dupe |

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

: spaces ;

本書建議儘量使用 `begin ... while ... repeat` 而不使用 `begin ... until`，因為使用後者常犯所謂差一的錯誤。

例子：

-------------------------------------
## 本章指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `if` | ( -- ) &emsp;  | if |
| `else` | ( -- ) &emsp;  | else |
| `then` | ( -- ) &emsp;  | then |
| `case` | ( -- ) &emsp;  | case |
| `endcase` | ( -- ) &emsp;  | endcase |
| `of` | ( -- ) &emsp;  | of |
| `endof` | ( -- ) &emsp;  | endof |
| `begin` | ( -- ) &emsp;  | begin |
| `while` | ( -- ) &emsp;  | while |
| `repeat` | ( -- ) &emsp;  | repeat |
| `until` | ( -- ) &emsp;  | until |
| `do` | ( -- ) &emsp;  | do |
| `loop` | ( -- ) &emsp;  | loop |
| `+loop` | ( -- ) &emsp;  | plus-loop |