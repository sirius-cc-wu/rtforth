# 循環 

## 不定循環 (Indefinite loop)

| x           | 0 | 15 | 30 | 45 | 60 | 75 | 90 |
|:------------|--:|---:|---:|---:|---:|---:|---:|
| sin(x)      | 0.000 | 0.259 | 0.500 | 0.707 | 0.866 | 0.966 | 1.000 |

```
\ 印出 n 個空格
: spaces ( n -- ) ;
```

```
\ 印出 sine table 的標頭
: .sin-header ( F: start end step -- )
( F: start end step )           frot
( F: end step start )           begin
( F: end step start )             fdup fround f>s 7 .r
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
測試一下：
```
rf> 0e 91e 15e .sin-header
      0     15     30     45     60     75     90 ok
```

```
\ 將角度轉成徑度
: deg ( n1 -- n2 ) 180e f/ pi f* ;

\ 印出 sine table 的值
: .sin-values ( F: start end step -- )
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

\ 印出 sine table
: .sin-table ( F: start end step -- )
   ( F: start end step )   2 fpick  2 fpick  2 fpick
   ( F: start end step start end step )   .sin-header  cr
   ( F: start end step )   .sin-values
;
```

```
rf> 0e 91e 15e .sin-table
      0     15     30     45     60     75     90
  0.000  0.259  0.500  0.707  0.866  0.966  1.000 ok
```

本書建議儘量使用 `begin ... while ... repeat` 而不使用 `begin ... until`，因為使用後者常犯所謂差一的錯誤。

### 中途結束

EXIT

### 本節指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `begin` | ( -- ) &emsp;  | begin |
| `while` | ( -- ) &emsp;  | while |
| `repeat` | ( -- ) &emsp;  | repeat |
| `until` | ( -- ) &emsp;  | until |
| `again` | ( -- ) &emsp;  | again |
| `.r` | ( -- ) &emsp;  | dot-r |
| `f.r` | ( -- ) &emsp;  | f-dot-r |

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
| `space` | ( -- ) &emsp;  | space |
| `spaces` | ( -- ) &emsp;  | spaces |
| `emit` | ( -- ) &emsp;  | emit |
| `[char]` | ( "c" -- ) &emsp;  | bracket-care |
