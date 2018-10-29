# 程式碼風格

---------
## 空格及縮排

指令和指令之間至少一個空格。如果一群指令構成一個有意義的片語，則片語和片語之間隔兩個空格。例如以下的 `1 counter +!` 被視為一個片語，

```forth
: up   activate  begin pause  1 counter +! again ;
```

指令名稱和它之後的定義之間空三格，如上例，或是將定義寫在下一行，縮排四個空格。如下，

```forth
: up ( n -- )
    activate
    begin
      pause  1 counter +!
    again
;
```

較長的巢狀控制結構縮排兩個空格，分號可以放在最後一個指令之後，或另起新的一行放在開頭，如上。

----------
## 註解

### 指令說明

很短的指令，如果很容易明瞭它的意思，可以不加指令說明。

較長或較不易理解的指令應加指令說明。指令說明以指令 `\` 開始，到同一行結尾。可以放在三個位置。

* 指令之前，例如，

```forth
\ 指令說明
: 指令 ( -- )   ... ;
```

* 和指令名稱同一行且在指令名稱之後，之間空一格，例如，

```forth
variable counter \ 計數值
: up \ ( n -- ) 指派工作 `n` 向上計數
    activate begin pause  1 counter +! again ;
```

* 指令內部，用以說明指令實作中較難理解的部份。例如，

```forth
: 指令 ( -- )
    \ 步驟一
    ...
    \ 步驟二
;
```

### 堆疊效果

所有的指令都需要有堆疊效果的註解。

指令的堆疊效果以 ( "input" before -- after ) 表示，"input" 代表指令會從輸入緩衝區取得的資料。
before 是指令會使用的資料堆疊上的資料，after 是指令執行後會放上堆疊的資料。

如果是浮點堆疊，會使用 ( F: before -- after ) 表示。

另外，使用以下字元來代表堆疊上資料的型別或意義：

* addr 或是 a：位址，程式碼空間或是資料空間的位址，或某個變數的位址。
* c：字元
* n：整數
* f：浮點數
* t：真假值

例如：

* `: ( "name" -- )` 冒號定義指令之後要跟著被定義指令的名稱。
* `+ ( n1 n2 -- n1+n2 )` 指令 `+` 會將堆疊上的兩個數字相加，並把和放上堆疊。
* `f+ ( F: f1 f2 -- f )` 將浮點數 `f1` 和 `f2` 相加，得到 `f`。
* `! ( n a -- )` 將整數 `n` 放進變數位址 `a`。
* `compiling? ( -- t )` 是否在編譯狀態？

如果指令的執行會依情況產生兩種不同的效果，使用 `|` 表示有兩種可能情況，例如，

```forth
: ?dup ( n -- 0 | n n )   dup if dup then  ;
```

若有必要，可以使用 `=` 表明堆疊上的資料的意義，或是不用等號，直接使用英文字表示，例如，

```forth
: game ( n1=seed -- | n2 ) ... ;
```

或是

```forth
: game ( seed -- | n2 ) ... ;
```

堆疊效果的註解放的位置有三處：

* 放在以指令 `\` 開始的註解內。例如

```forth
\ 簡短的說明 ( n n -- )
\ 更清楚的說明
: 指令 .... ;
```

* 放在指令定義之後，和指令名稱之間空一個空格，和之後的程式空三個空格。例如，

```forth
: 2dup ( n1 n2 -- n1 n2 n1 n2 )   over over ;
```

* 放在複雜的指令內部說明堆疊的變化，例如，

```forth
: game ( n1=seed -- | n2 )
   ?dup if
      xorshift  100 mod  abs  ( n2 )
   else
      ." 種子不可以為 0"        ( )
   then ;
```
或

```forth
: .sin-table ( F: start end step -- )
   ( F: start end step )   2 fpick  2 fpick  2 fpick
   ( F: start end step start end step )   .sin-header  cr
   ( F: start end step )   .sin-values
;
```

----------
## 命名原則

指令的名稱不區分大小寫。動程科技在本手冊中使用小寫。

以下是動程科技撰寫 Forth 程式時指令的命名原則，也是本手冊指令的命名原則。

命名 | 意義 | 書中範例
----|------|-----
!name | 存入一整筆資料 |
name! | 存入一項資料 | handler!
@name | 取出一整筆資料 |
name@ | 取出一項資料 |
#name | 資料總數 |
name# | 號碼、代號 | yellow#
'name | 資料位址 |
(name) | 實作 name 的內部指令 | (abort)
+name | 增加   | +field
      | 致能、啟用 |
-name | 減少、移除 | -work
      | 禁用，使停止 |
.name | 印出資料 | .error .token .row
      | 以某種方式印出 | .r f.r
name. | 印出某種型別的資料 | h. f.
;name | 結束 |
?name | 有條件的執行 | ?dup ?do
      | 檢查 | ?stacks
name? | 狀況，回傳真假值 | compiling? token-empty?
/name | 資料的位元組數 | /point /person
>name | 轉換成 name |
name> | 轉換自 name |
name, | 編譯進字典 | 2, f,
0name | 清除，重設 | 0error 0stacks
2name | 兩筆資料 | 2, 2dup
name-name | 複合字使用 - 連接 | token-empty?
