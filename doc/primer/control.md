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

## 猜數字

現在我們來設計一款猜數字的遊戲。

遊戲一開始會先產生一個比 100 小的數字藏在堆疊上，當我們猜一個數字時，遊戲會和堆疊上的數字作比較，如果太大就會說「太大」，太小就說「太小」。猜中了就說「答對了」。

以下是玩遊戲的一個案例：

```
rf> game
rf> 32 guess
太小
rf> 64 guess
太大
rf> 54 guess
答對了!
```

我們設計兩個 Forth 指令：
* 指令 `game` 會產生一個比 100 小的數字放上堆疊。因此它的堆疊效果是 ( -- n )。
* 指令 `guess` 會比較堆疊上的兩個數，一個是 `game` 產生的，另一個是玩家猜的。如果猜的數比較大，就印出「太大」，太小就印「太小」，這兩種情況下，那個隱藏在堆疊上的數字都應繼續藏在堆疊上。當猜中時，印出「答對了」，而且把藏在堆疊上的數字拋棄。因此堆疊效果是 ( n1 n2 -- | n1 ) 。在此我們使用垂直線表示堆疊的兩種情況。

為了要產生一個小於 100 的數字，我們需要一個亂數產生器。某些 Forth 版本會提供產生亂數的指令。在此我們使用 George Marsaglia 的
[Xorshift 亂數產生器](https://en.wikipedia.org/wiki/Xorshift)。
```
: xorshift ( n -- x ) dup 13 lshift xor dup 17 rshift xor dup 5 lshift xor ;
```
使用這產生器，我們需要先給一個亂數種子 (seed) ，`xorshift` 每次執行時會利用這種子計算出一個新的數字，並且把這新的數字當成新的種子。在我們尚未學到變數 (variable) 前，先把種子放在堆疊上。未來談到變數時，再將這個種子放在變數中，就不必每次都要讓玩家輸入了。

測試一下：

```
rf> : xorshift ( n -- x ) dup 13 lshift xor dup 17 rshift xor dup 5 lshift xor ;
 ok
rf> 1 xorshift .s
270369  ok
rf> xorshift .s
68787111425  ok
rf> xorshift .s
18597760640231621  ok
rf> xorshift .s
5629809312759907  ok
rf> xorshift .s
-8956027557026519269  ok
rf> xorshift .s
9011377231533407587  ok
rf> xx
xx Undefined word
```
最後我們下一個 Forth 不認識的指令 `xx` ，使得它清除堆疊。

經過幾次測試，我們發現亂數的種子不可以是 0。
```
rf> 0 xorshift .s
0  ok
```

因此我們修改一下 `game` 的規格：

* 指令 `game`：玩家先在堆疊上放一個他喜歡的數字。這個數字不可以是 0。指令 `game` 會以 `xorshift` 產生一個比 100 小的數字放上堆疊。因此它的堆疊效果是 ( n1=seed -- n2 )，在此我們使用 n1=seed 來表示 n1 是一個種子。

因為 `xorshift` 產生的數字可能超過 100，我們可以使用求餘數的指令 `mod` 來得到小於 100 的數字。但是必須注意到 `xorshift` 可能會產生小於 0 的數字。而不同 Forth 版本的指令 `mod` 在除數和被除數的正負號不同時，會有不同的答案。請看以下 rtForth、SwiftForth 和 gforth 的測試結果：

rtForth:
```
rf> -100 9 mod .
-1  ok
```
SwiftForth:
```
-100 9 mod . -1  ok
```
gforth:
```
-100 9 mod . 8  ok
```

解決的方法是取 `mod` 結果的絕對值。

以下是我們對 `game` 的定義：

```
\ Start the guessing game, n1 is a seed to generate
\ the hidden number n2 on stack. N2 should be positive and less than 100. 
\ If the seed given is zero, print an error message and drop the seed.
: game ( n1=seed -- | n2 )
   dup if
      xorshift  100 mod  abs  ( n2 )
   else
      ." 種子不可以為 0" drop   ( )
   then ;
```
請分析一下每行的堆疊效果以確定你瞭解這段程式。

以下是我們對 `guess` 的定義：
```
\ 當 n2 > n1，印出「太大」。當 n2 < n1，印出「太小」。這兩種情況都保留 n1 在堆疊上。否則印出「答對了」，並拋棄 n1。
: guess ( n1 n2 -- | n1 )
   2dup < if ." 太大"  drop
   else 2dup >
      if ." 太小"  drop
      else ." 答對了"  2drop
      then
   then ;
```
請分析一下每行的堆疊效果以確定你瞭解這段程式。

玩一下我們的遊戲：
```
rf> 10 game
 ok
rf> 8 guess
太小 ok
rf> 20 guess
太小 ok
rf> 50 guess
太小 ok
rf> 80 guess
太小 ok
rf> 90 guess
答對了 ok
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