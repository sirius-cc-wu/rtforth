# 字典

TODO

Forth 能執行 `+` 、 `-` 、`*` 、 `/` 這些指令，因為它的系統內有一部字典 (wordlist).。
可以使用 `words` 檢查 Forth 字典內有哪些字。試一下：

```
rf> words

(abort) quit evaluate accept 1/sec hz rpm um/msec mm/min usec msec sec minute
hr rad deg um mm meter fnegate fceil fround floor fmax fmin f< f0= f0< f~ f**
f/ f* f- f+ f>s s>f fover frot fnip fswap fdup fdrop fsqrt fatan2 fatan facos
fasin fsincos ftan fcos fsin fabs f@ f! fvariable fconstant utime ntime max-u
max-n .s words flush f. . .( ." s" type emit interpret-token compile-token
token-empty? compiling? bye abort reset handle-error error? handler! marker ,
] ' create variable constant : parse char parse-word between max min negate abs
mod / 2over 2swap 2drop 2dup rot <> > 0<> 0> 0= not false true +loop loop ?do
do recurse again repeat while begin then else if ; [char] [ \ ( base c! c@
allot here chars char+ ! @ cells cell+ /mod * + - 1- 1+ rshift lshift xor or
and invert < = 0< ?stacks depth nip over swap drop dup execute 2r@ 2r> 2>r r@
r> >r j i leave unloop _+loop _loop _qdo _do 0branch branch _s" flit lit halt
exit noop  ok
```

字典如何記載一個指令？

-----------
## 常數、變數
像 `true` 和 `false` 這類被賦與固定數值的指令，被稱為常數。Forth 定義整數常數的方法如下：
```
<整數> constant <常數名>
```

例子：
```
variable 狀態  (　定義一個名為「狀態」的整數變數 )
binary  ( 切換到二進制 )
001 constant 冷氣  ( 以右邊數來第一個位元代表冷氣開關 )
010 constant 風扇  ( 以右邊數來第二個位元代表風扇開關 )
100 constant 冰箱 ( 以右邊數來第三 個位元代表風扇開關 )
111 constant 全部
11　狀態　!        ( 目前的狀態是冷氣和風扇都開著、冰箱關著 )
decimal  ( 切換回二進制 )
冷氣 風扇 or　狀態　!        ( 目前的狀態是冷氣和風扇都開著、冰箱關著 )
冷氣  狀態　@  and  .  ( 檢查冷氣是否開著 )
風扇  狀態　@  and  .  ( 檢查風扇是否開著 )
冰箱  狀態　@  and  .  ( 檢查冰箱是否開著 )
冷氣 冰箱 or  狀態　@ 冰箱　and  .  ( 檢查冷氣或是冰箱中是否至少有一個開著 )
狀態　@ 冰箱　冷氣 or and  .  ( 檢查風扇是否開著 )
全部  狀態 @  =  ( 是否全部開著 )
狀態 @  冷氣 xor  全部 and  狀態 !
```
### 本節指令集
| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `constant` | ( -- ) &emsp; | constant |
| `variable` | ( -- ) &emsp; | variable |
| `fconstant` | ( -- ) &emsp; | f-constant |
| `fvariable` | ( -- ) &emsp; | f-variable |

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