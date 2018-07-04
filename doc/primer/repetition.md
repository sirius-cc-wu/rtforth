# 循環 

Forth 有多種表達循環的方式，

* 不定循環 (Indefinite loop) - 知道循環停止條件時使用。
  * `begin` `while` `repeat` - 在每次循環中測試終止條件
  * `begin` `until` - 在每次循環之後進行測試終止條件
  * `begin` `again` - 無限循環
* 定循環 (Definite loop) - 知道循環次數或上限時使用。
  * `do` `loop` 或 `?do` `loop` - 每次循環計數值加一
  * `do` `+loop` 或 `?do` `+loop` - 每次循環計數值加上堆疊上的整數

這些表達循環的指令和 `if` 等指令一樣，都是「編譯指令」。

## 不定循環 (Indefinite loop)

我們先看一個能印出指定數量空格的指令 `spaces` 的定義：
```
: spaces ( n -- )   0 begin 2dup > while 1+ space repeat 2drop ;
```

指令 `spaces` 使用已有的指令 `space` 印出空格，並使用編譯指令 `begin` 、 `while` 、 `repeat` 實現重覆印出的行為。

我們以直式並加上堆疊註解的方式分析這個指令，以理解 `begin` 、 `while` 、 `repeat` 的作用。

```
\ 印出 n 個空格
: spaces ( n -- )
  ( n )        0        \ 在堆疊上放 0，代表目前印出的空格數是 0
  ( n 0 )      begin    \ 標記了不定循環的開始
  ( n 0 )        2dup   \ 複製堆疊上的兩個數字
  ( n 0 n 0 )    >      \ 並進行比較
  ( n 0 flag ) while    \ 當 n 大於已印出的空格數時
  ( n 0 )        1+     \   已印出的空格數加 1
  ( n 1 )        space  \   印出一個空格
  ( n 1 )      repeat   \   重覆執行 begin 開始的敘述
  ( n 1 )      2drop    \ 否則離開循環，拋棄堆疊上的兩個數
  ( )          ;
```

指令 `begin` 標記了不定循環的開始，`while` 則測試堆疊上的數字，如果是 0 就結束循環，`repeat` 重覆從 `begin` 開始的不定循環。

以下是句子 `begin A while B repeat C` 編譯結果的示意圖：

```
begin A while B repeat C

                +--------------------+
                |                    |
                |                    v
+---+---------+---+---+--------+---+---+
| A | 0branch | x | B | branch | x | C |
+---+---------+---+---+--------+---+---+
  ^                              |
  |                              |
  +------------------------------+
```

例子：以不定循環指令印出以下表格。

| x           | 0.000 | 15.000 | 30.000 | 45.000 | 60.000 | 75.000 | 90.000 |
|:------------|--:|---:|---:|---:|---:|---:|---:|
| sin(x)      | 0.000 | 0.259 | 0.500 | 0.707 | 0.866 | 0.966 | 1.000 |

首先我們定義一個指令 `.sin-header` 印出第一行，
```
\ 印出 sine table 的標頭。
\ 浮點堆疊上有三個數
\ start: 開始的角度
\ end: 結束的角度
\ step: 增量
\ 印出標頭會是 start start+step start+2*step ... 直到大於 end 為止。
: .sin-header ( F: start end step -- )
( F: start end step -- )        ." x      "
( F: start end step )           frot
( F: end step start )           begin
( F: end step start )             fdup  7 3 f.r
( F: end step start )             fover f+
( F: end step start' )            frot
( F: step start' end )            fover fover
( F: step start' end start' end ) f> not
( flag F: step start' end )     while
( F: step start' end )            frot frot
( F: end step start' )          repeat
( F: step start' end )          fdrop fdrop fdrop
;
```

以上定義中有一個未提過的指令 `f.r`，這個指令會依據資料堆疊上的兩個數來印出浮點堆疊上的浮點數。資料堆疊上的這兩個數，一個代表印出的欄寬。另一個代表小數點後的位數。以上定義中的 `7 3 f.r` 會印出欄寬為 7 、小數點後有三位數，向右對齊的數字。指令 `f.r` 和之前的 `f.` 還有一項不同： `f.r` 不會在最後多印出一個空格。指令 `f.r` 讓我們能對齊不同大小的浮點數。

rtForth 的指令 `f.` 是使用 `f.r` 定義出來的。以下是它的定義：
```
: f. ( F: r -- )   0 7 f.r space ;
```

同樣的，有個和 `f.r` 類似，但用於整數，可以指定欄寬並向右對齊的指令 `.r`。在 rtForth 中的 `.` 是以 `.r` 定義出來的。
```
: . ( n -- )   0 .r space ;
```

請確實理解 `.sin-header` 的定義，確定每一個浮點堆疊註解合乎預期。並測試一下，
```
rf> 0e 91e 15e .sin-header
x        0.000 15.000 30.000 45.000 60.000 75.000 90.000 ok
```

接下來我們設計指令印出第二排的正弦值。

計算正弦的指令 `fsin` 的參數是徑度，我們必須把角度轉成徑度。rtForth 提供指令 `deg` 進行這項轉換。其他 Forth 系統可以定義 `deg` 如下：
```
\ 將角度轉成徑度
: deg ( n1 -- n2 ) 180e f/ pi f* ;
```

以下是印出正弦值的指令 `.sin-values` 的定義。它和 `.sin-header` 幾乎一模一樣，只差在 `." x"` 被改成 `." sin(x)"`。以及 `fdup  7 3 f.r` 被改成 `fdup deg fsin  7 3 f.r` 。

```
\ 印出 sine table 的值
: .sin-values ( F: start end step -- )
( F: start end step )        ." sin(x) "
( F: start end step )           frot
( F: end step start )           begin
( F: end step start )             fdup deg fsin  7 3 f.r
( F: end step start )             fover f+
( F: end step start' )            frot
( F: step start' end )            fover fover
( F: step start' end start' end ) f> not
( flag ) ( F: step start' end ) while
( F: step start' end )            frot frot
( F: end step start' )          repeat
( F: step start' end )          fdrop fdrop fdrop
;
```

最後我們使用 `.sin-header` 和 `.sin-values` 來合成印表格的 `.sin-table` 指令。如下。這兒使用了我們之前未提及的一個浮點堆疊操作指令 `fpick` 。指令 `fpick` 會依整數堆頂的整數來複製對應的浮點堆疊內的整數。敘述 `0 fpick` 的行為和 `fdup` 一樣，會複製浮點堆疊從疊頂數來的第一個浮點數。敘述 `1 fpick` 的行為和 `fover` 一樣，會複製浮點堆疊從疊頂數來的第二個浮點數。`2 fpick` 則複製疊頂第三個浮點數。在此使用 `fpick` 是因為我們需要複製疊頂數來第三個浮點數。這是之前其他浮點堆疊運算指令做不到的。

```
\ 印出 sine table
: .sin-table ( F: start end step -- )
   ( F: start end step )   2 fpick  2 fpick  2 fpick
   ( F: start end step start end step )   .sin-header  cr
   ( F: start end step )   .sin-values
;
```

測試一下，

```
rf> 0e 91e 15e .sin-table
x        0.000 15.000 30.000 45.000 60.000 75.000 90.000
sin(x)   0.000  0.259  0.500  0.707  0.866  0.966  1.000 ok
```

不定循環中的另兩個版本 `begin ... until` 和 `begin ... again` 可視為 `begin ... while ... repeat` 的簡化版。
敘述 `begin ... ( flag ) until` 會在 `until` 比較資料堆疊上的數是否為真，若為真就結束循環。因此它的行為和 `begin ... ( flag ) 0= while repeat` 一樣。注意在這兒 `while` 和 `repeat` 之間不執行任何指令。而無限循環 `begin ... again` 則相當於敘述 `begin ... true while repeat` 。

本書建議儘量使用 `begin ... while ... repeat` 不使用 `begin ... until`，因為使用後者常犯所謂差一的錯誤，忘了對循環的第一次執行進行條件測試。例如以下使用 `until` 實現的 `spaces` 因忘了檢查 n=0 的情形，導致當 n 為 0 時仍印出了一個空格。

: wrong-spaces ( n -- )   0 begin space 1+ 2dup <= until 2drop ;

### 中途結束

EXIT

### 本節指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `space` | ( -- ) &emsp;  | space |
| `spaces` | ( -- ) &emsp;  | spaces |
| `begin` | ( -- ) &emsp;  | begin |
| `while` | ( -- ) &emsp;  | while |
| `repeat` | ( -- ) &emsp;  | repeat |
| `until` | ( -- ) &emsp;  | until |
| `again` | ( -- ) &emsp;  | again |
| `.r` | ( -- ) &emsp;  | dot-r |
| `f.r` | ( -- ) &emsp;  | f-dot-r |
| `fpick` | ( -- ) &emsp;  | fpick |

## 定循環 (Definite loop)

```
: spaces ;
```
指令 `space` 和 `spaces` 都是 Forth 2012 標準內的指令。

```
rf> : star [char] * emit ;
rf> star
* ok
rf> : stars 0 do star loop ;
 ok
rf> 5 stars
***** ok
```

```
rf> 0 stars
* ok
```

```
rf> : stars 0 ?do star loop ;
Redefining stars ok
rf> 0 stars
 ok
```

### 兩重循環

```
: .table cr 10 1 do 10 1 do i j * 5 .r loop cr loop ;
```

例子：
```
rf> : .table cr 10 1 do 10 1 do i j * 5 .r loop cr loop ;
 ok
rf> .table

    1    2    3    4    5    6    7    8    9
    2    4    6    8   10   12   14   16   18
    3    6    9   12   15   18   21   24   27
    4    8   12   16   20   24   28   32   36
    5   10   15   20   25   30   35   40   45
    6   12   18   24   30   36   42   48   54
    7   14   21   28   35   42   49   56   63
    8   16   24   32   40   48   56   64   72
    9   18   27   36   45   54   63   72   81
 ok
```

### 印出所有 3 的倍數

```
rf> : .multiple3   0 do i . 3 +loop ;
 ok
rf> 17 .multiple3
0 3 6 9 12 15  ok
```

### 中途結束

LEAVE

```
rf> : wrong-loop  0 do 42 emit  i 5 > if exit then loop ;
 ok
rf> 7 wrong-loop
*******Undefined word
```

UNLOOP

```
rf> : correct-loop   0 do 42 emit  i 5 > if unloop exit then loop ;
 ok
rf> 7 correct-loop
******* ok
```

### 本節指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `do` | ( -- ) &emsp;  | do |
| `?do` | ( -- ) &emsp;  | question-do |
| `loop` | ( -- ) &emsp;  | loop |
| `+loop` | ( -- ) &emsp;  | plus-loop |
| `leave` | ( -- ) &emsp;  | leave |
| `unloop` | ( -- ) &emsp;  | unloop |
| `i` | ( -- ) &emsp;  | i |
| `j` | ( -- ) &emsp;  | j |
| `emit` | ( -- ) &emsp;  | emit |
| `[char]` | ( "c" -- ) &emsp;  | bracket-care |
