# 指令集

Forth 能執行 `+` 、 `-` 、`*` 、 `/` 這些指令，是因為它的系統內建的指令集 (word list).。
指令 `words` 會顯示出 Forth 指令集內的指令。

```
rf> words

empty (abort) quit evaluate >in source tib #tib fill c, min
max +! 2variable 2! 2@ align aligned spaces space bl cr
?dup f> >= <= h. hex decimal accept 1/sec hz rpm um/msec
mm/min usec msec sec minute hr rad deg um mm meter fnegate
fceil fround floor fmax fmin f< f0= f0< f~ f** f/ f* f- f+
f>s s>f fover frot fnip fswap fdup fdrop fsqrt fatan2 fatan
facos fasin fsincos ftan fcos fsin fabs f@ f! pi fvariable
...
```

指令集記載了指令的名稱、行為、資料，並提供搜尋指令的方法。

```
指令
            +-----------------+
            | +               | 名稱
            +-----------------+
            | 將堆疊上的數字相加 | 行為
            +-----------------+
            | 無              | 資料
            +-----------------+

指令集
            +--------+      +---------+      +------+      +----------+
  LAST ---> | empty  | ---> | (abort) | ---> | quit | ---> | evaluate | --->
            +--------+      +---------+      +------+      +----------+
            | unmark |      | nest    |      | nest |      | nest     |
            +--------+      +---------+      +------+      +----------+
            |        |      |         |      |      |      |          |
            +--------+      +---------+      +------+      +----------+
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