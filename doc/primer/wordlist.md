# 指令集

## 定義第一個指令
Forth 能執行 `+` 、 `-` 、`*` 、 `/` 這些指令，是因為它的系統內建的指令集 (word list).。
指令 `words` 會顯示出 Forth 指令集內的指令。

```
rf> words

empty (abort) quit evaluate >in source tib #tib fill c, min
max +! 2variable 2! 2@ align aligned spaces space bl cr
?dup f> >= <= h. hex decimal accept 1/sec hz rpm um/msec
mm/min usec msec sec minute hr rad deg um mm meter fnegate
fceil fround floor fmax fmin f< f0= f0< f~ f** f/ f* f- f+
...
```

指令集記載了指令的名稱、行為、資料，並提供搜尋指令的方法。

```
指令
            +------+
        名稱 |  +   |
            +------+      +-----------------+
        行為 | nest | ---> | 將堆疊上的數字相加 |
            +------+      +-----------------+
        資料 | 無   |
            +------+

指令集
            +--------+      +---------+      +------+      +----------+
  LAST ---> | empty  | ---> | (abort) | ---> | quit | ---> | evaluate | --->
            +--------+      +---------+      +------+      +----------+
            | unmark |      | nest    |      | nest |      | nest     |
            +--------+      +---------+      +------+      +----------+
            |        |      |         |      |      |      |          |
            +--------+      +---------+      +------+      +----------+
```

在以上的示義圖中，LAST 指的是最後一個定義的指令。指令 `words` 先顯示較晚定義的指令，再顯示較早定義的指令。以之前的例子來看，指令 `empty` 是最後一個定義的指令，再來是 `(abort)`，再來是 `quit`。

現在讓我們定義本書的第一個指令。這個指令在安裝的章節已經定義過。輸入時請注意 Forth 使用空格來分開指令，因此指令 `."` 之後要有空格，指令 `;` 之前也要有空格：

```
rf> : hello ." Hello World!" ;
 ok
rf> words

hello empty (abort) quit evaluate >in source tib #tib fill
c, min max +! 2variable 2! 2@ align aligned spaces space bl
...
```

可以看到 `words` 顯示的最後一個指令是剛才定義的 `hello` 。這個指令是使用冒號 (:) 定義出來的。因此被稱為冒號定義指令 (colon definition)。冒號定義指令相當於其他程式語言的副程式或函式。

冒號 `:` 定義了一個新指令，在它之後的 hello 是這個新指令的名稱。名稱之後到 `;` 之前的部份是這個指令的行為。指令 `;` 結束了這新定義定義。

```
指令
            +-------+
        名稱 | hello |
            +-------+      +------------------+
        行為 | nest  | ---> | ." Hello World!" |
            +-------+      +------------------+
        資料 | 無    |
            +-------+
```

測試一下新定義的指令。

```
rf> hello
Hello World! ok
```


```
rf> empty
 ok
rf> words

(abort) quit evaluate >in source tib #tib fill c, min max
+! 2variable 2! 2@ align aligned ...
```

```
rf> marker empty
 ok
rf> words

empty (abort) quit evaluate >in source tib #tib fill c, min
max +! 2variable 2! 2@ align aligned ...
```

```
rf> : hello  ." Hello World!" ;
 ok
rf> hello
Hello World! ok
rf> empty
 ok
rf> hello
hello Undefined word
```

### 本節指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `words` | ( -- ) &emsp;  | words |
| `empty` | ( -- ) &emsp;  | empty |
| `marker` | ( -- ) &emsp;  | marker |
| `:` | ( -- ) &emsp; | colon |
| `;` | ( -- ) &emsp; | semicolon |
| `."` | ( -- ) &emsp; | dot-quote |

-----------
## 常數、變數
像 `true` 和 `false` 這類被賦與固定數值的指令，被稱為常數。Forth 定義整數常數的方法如下：
```
<整數> constant <常數名>
```

例子：
```
variable 狀態  (　定義一個名為「狀態」的整數變數 )
%001 constant 冷氣  ( 以右邊數來第一個位元代表冷氣開關 )
%010 constant 風扇  ( 以右邊數來第二個位元代表風扇開關 )
%100 constant 冰箱 ( 以右邊數來第三 個位元代表風扇開關 )
%111 constant 全部
%11　狀態　!        ( 目前的狀態是冷氣和風扇都開著、冰箱關著 )
冷氣 風扇 or　狀態　!        ( 目前的狀態是冷氣和風扇都開著、冰箱關著 )
冷氣  狀態　@  and  .  ( 檢查冷氣是否開著 )
風扇  狀態　@  and  .  ( 檢查風扇是否開著 )
冰箱  狀態　@  and  .  ( 檢查冰箱是否開著 )
冷氣 冰箱 or  狀態　@ and  .  ( 檢查冷氣或是冰箱中是否至少有一個開著 )
全部  狀態 @  =  ( 是否全部開著 )
```
### 本節指令集
| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `constant` | ( -- ) &emsp; | constant |
| `variable` | ( -- ) &emsp; | variable |
| `fconstant` | ( -- ) &emsp; | f-constant |
| `fvariable` | ( -- ) &emsp; | f-variable |

-------------
## 本章重點整理

* 指令集 (word list)

-----------
## 本章指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|------------------------------------|--------|
| `words` | ( -- ) &emsp;  | words |
| `marker` | ( -- ) &emsp; | marker |
| `empty` | ( -- ) &emsp; | empty |
| `:` | ( -- ) &emsp; | colon |
| `;` | ( -- ) &emsp; | semicolon |
| `constant` | ( -- ) &emsp; | constant |
| `variable` | ( -- ) &emsp; | variable |
| `fconstant` | ( -- ) &emsp; | f-constant |
| `fvariable` | ( -- ) &emsp; | f-variable |
| `create` | ( -- ) &emsp; | create |
| `cells` | ( -- ) &emsp; | cells |
| `cell+` | ( -- ) &emsp; | cell+ |
| `align` | ( -- ) &emsp; | align |
| `aligned` | ( -- ) &emsp; | aligned |
| `allot` | ( -- ) &emsp; | allot |
| `here` | ( -- ) &emsp; | here |
| `,` | ( -- ) &emsp; | comma |
| `does>` | ( -- ) &emsp; | does |
| `@` | ( -- ) &emsp; | fetch |
| `!` | ( n a -- ) &emsp; 將 n 存在位址 a  | store |
| `2@` | ( -- ) &emsp; | two-fetch |
| `2!` | ( -- ) &emsp; | two-store |
| `+!` | ( n a -- ) &emsp; 將位址 a 內的整數加 n | plus-store |
| `f@` | ( -- ) &emsp; | f-fetch |
| `f!` | ( -- ) &emsp; | f-store |