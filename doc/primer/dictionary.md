# 字典

Forth 系統核心的資料結構有二：

* 堆疊：參數堆疊、浮點堆疊，以及本書未深入提及的返回堆疊。指令和指令之間的資料透過堆疊傳遞。
* 字典：記錄 Forth 所有的指令。提供文本解譯器查尋及執行指令的功能。

在之前的章節，我們花了許多篇幅熟悉堆疊的操作。以及熟悉眾多指令中的一種：冒號定義指令。在這章我們把重點放在字典，在字典中增加新的指令的方法，以及指令存放在字典中的方式。

想在字典中增加新的指令，可以使用「定義指令」。冒號就是一種定義指令。以下是我們將在這章學到的定義指令：

* 定義字典標記： `marker`
* 定義常數：`constant` 、 `2constant` 、 `fconstant`
* 定義變數：`variable` 、 `2variable` 、 `fvariable`
* 定義資料結構：`create` 、 `+field`

我們也將學習如何使用字典中的資料空間。

以下範例使用 32 位元的 rtForth v0.6.0。如果您使用的是 64 位元的 rtForth 或其他版本的 rtForth，記憶體位址的數值會有很大的差異。

## 指令 words

Forth 能執行 `+` 、 `-` 、`*` 、 `/` 這些指令，是因為它內建的字典 (dictionary) 提供了搜尋及執行指令的功能。如果我們想知道字典中有多少指令，我們可以執行指令 `words`。

```
rf> words

-work cold (abort) quit evaluate-input ms release get stop
halt nod operator xtime dump _dump _type >char bounds count
pad >in source tib #tib +field fvariable 2variable 2constant
does> variable fill c, chars min max 2, +! 2! 2@ f, cr ?dup
f> >= <= h.r h. hex decimal ? f. . spaces space bl bye receive
...
```

字典記載了指令的名稱、行為、資料，並提供搜尋指令的方法。

```
指令
            +------+
        名稱 |  +   |
            +------+      +-----------------+
        行為 | plus | ---> | 將堆疊上的數字相加 | 程式碼空間
            +------+      +-----------------+
        資料 | 無   |
            +------+

字典
            +--------+      +---------+      +------+      +----------+
  LAST ---> | -work  | ---> | (abort) | ---> | quit | ---> | evaluate | --->
            +--------+      +---------+      +------+      +----------+
            | unmark |      | nest    |      | nest |      | nest     |
            +--------+      +---------+      +------+      +----------+
            |        |      |         |      |      |      |          |
            +--------+      +---------+      +------+      +----------+
```

指令 `words` 先顯示較晚定義的指令，再顯示較早定義的指令。以之前的例子來看，指令 `-work` 是最後一個定義的指令，再來是 `(abort)`，再來是 `quit`。示義圖中，LAST 指的是最後一個定義的指令，也就是 `-work`。它是一個之後會提到的字典標記指令，執行它時，它的行為是 `unmark`。其他的幾個指令的行為是 `nest`。行為是 `nest` 的指令就是我們之前提到的冒號定義指令。

現在讓我們定義本書的第一個指令 `hello`。看看字典有什麼變化。

```
rf> : hello ." Hello World!" ;
 ok
rf> words

hello -work cold (abort) quit evaluate-input ms release get
stop halt nod operator xtime dump _dump _type >char bounds
...
```

可以看到 `words` 顯示的最後一個指令是剛才定義的 `hello`。

```
: hello   ." Hello World!" ;

            +-------+                                  +-------+
LAST -> 名稱 | hello | -------------------------------> | -work | ---> 
            +-------+        +---------------+         +-------+
        行為 | nest  | -----> | 執行冒號定義指令 | 程式碼空間
            +-------+        +---------------+
        資料 |       | --+
            +-------+   |    +-----+----+--------------+------+------+
                        +--> | _s" | 12 | Hello World! | type | exit | 資料空間
                             +-----+----+--------------+------+------+
```

從圖中可以看出指令的行為被放在程式碼空間 (code space) 中，資料則放在資料空間 (data space) 中。
這兒所謂的空間，就是一塊連續的記憶體。程式碼空間放的是可以執行的機器語言程式碼。資料空間中放的則是機器語言程式碼執行時所需的資料。某些系統裡將機器語言程式碼和資料放在同一個空間裡。在另外一些系統中，存放名稱、行為、資料的那個資料結構被放在另一個名為標頭空間 (header space) 的記憶體中。

上圖中的 `nest` 對應的是執行冒號定義指令的機器語言程式碼。它會取得資料空間中的 Forth 指令依順序執行。第一個指令 `_s"` 會將長度為 12 的字串 "Hello World!" 的開始位置和長度放在堆疊上。其後的 `type` 則會依據堆疊上的位置和長度印出字串的內容。這兒的 `_s"` 和其後的數字和字串，以及 `type` ，都是编譯指令 `."` 编譯進字典中的。例中顯示的只是 rtForth 的編譯結果。不同的 Forth 系統有不同的编譯方式。指令 `_s"` 不是 Forth 2012 的標準指令。但是 `type` 是一個標準指令。

另有一個編譯指令 `s"` 類似 `."` 但只編譯上例中的 `_s"` 及其後字串的部份，不編譯 `type`。因此以下兩個冒號定義指令的效果是一樣的：

```
: hello-1   ." Hello World!" ;
: hello-2   s" Hello World!" type ;
```

RtForth 的指令 `.memory` 列出 rtForth 對程式碼空間和資料空間的使用情形，

```
rf> .memory
code space capacity: 4194304, used: 0, start: 0x7F9520C00000, limit: 0x7F9521000000, here: 0x7F9520C00000
data space capacity: 4194304, used: 5344, start: 0x7F9521000000, limit: 0x7F9521400000, here: 0x7F95210014E0
```

在上面的例子中，程式碼和資料空間容量各為 4Mb。程式空間目前沒被使用，資料碼空間使用了 5344 個位元。

### 本節指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `words` | ( -- ) &emsp; 顯示字典中目前能使用的指令 | words |
| `s" <string>"` | ( -- ) &emsp;  編譯其後的字串直到下一個 "。 | s-quote |
| `type` | ( addr n -- ) &emsp; 印出資料空間位置 addr 處，長度為 n 的字串 | type |
| `.memory` | ( -- ) &emsp; 印出資料空間和程式碼空間的使用情況。 | dot-memory |

-------------------
## 標記指令 (Marker)

當你使用 rtForth，不想保留自己定義的指令時，可以執行 `-work。
```
rf> -work
 ok
rf> words

(abort) quit evaluate >in source tib #tib fill c, min max
+! 2variable 2! 2@ align aligned ...
```
你會發現，不只是剛剛定義的 `hello` 不見了，連 `-work` 也不見。

指令 `-work` 標記了字典的特定位置 (包括定義 -work 時資料空間和程式碼空間的位置)。執行時，會丟棄指令本身以及其後的所有指令，並且歸還這些指令使用的資料空間和程式碼空間。

你可以使用 `marker` 定義這類標記指令。

```
rf> marker -work
 ok
rf> words

-work (abort) quit evaluate >in source tib #tib fill c, min
max +! 2variable 2! 2@ align aligned ...
```

於是一個新的 `-work 又出現了。

指令 `-work` 並不是 Forth 2012 的標準指令。但是 `marker` 是一個標準指令。它的主要用處是標記字典的特定位置，以便在必要時移除一群指令。

### 例子：CNC 工具機

CNC 工具機內有兩種不同類型的程式：PLC 程式，也就是所謂的階梯圖，處理機器內各種裝置的開關，以及工件程式，也就是所謂的 G 代碼，處理馬達的運動和加工行為。這兩種不同的程式，都可以透過編譯器翻譯成 Forth 指令。例如 PLC 程式可以翻譯成一個 Forth 指令 `run-plc`。工件程式可以翻譯成 Forth 指令 `run-nc`。這兩個指令都放在字典中。

Forth 本身是一個多工的系統。因此可以使用一個 Task 執行 `run-plc`，另一個 Task 執行 `run-nc`。下一章會說明 rtForth 的多工環境。

一台機器的 PLC 程式通常是不變的，開機時就載入的。但機台製造商的維護的人員有權限修改它。而工件程式則是操作員選擇後才載入的，很可能一天要換好幾個工件程式。因此我們可以用 marker 對字典進行以下的規畫：

Forth 程式一：定義不隨工件程式和 PLC 變動的指令。在開機時載入。
```
\ 定義不隨 PLC 和工件程式變動的指令。
: xxx ... ;

\ 定義 PLC 和 工件程式的 markers。一開始 PLC 程式和工件程式都是空的。
marker -plc
marker -nc
```

Forth 程式二：PLC 程式。在開機時載入，或在維護人員修改 PLC 程式時載入。
```
\ 清除之前的 PLC 程式
-plc   marker -plc

: run-plc   ... ;

\ 定義工件程式的 markers。目前工件程式是空的。
marker -nc
```

Forth 程式三：工件程式，在操作員選擇加工程式時載入。
```
-nc   marker -nc

: run-nc   ... ;
```

如此，當維護人員要修改 PLC 程式時，指令 `-plc` 會清除 PLC 程式以及之後的工件程式。並定義新的 `-plc` 和 `-nc` 這兩個 markers。當操作員要下載新的工件程式時，指令 `-nc` 會清除舊的工件程式，並定義新的 `-nc` marker。

### 本節指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `marker <name>` | ( -- ) &emsp; 定義一個用來刪除一群指令，名為 `name` 的標記指令。執行此一指令時，會刪除自己以及其後定義的所有指令，並釋放所有這些指令配置的記憶體。 | marker |

-----------
## 常數
像 `true` 、 `false` 和 `pi` 這類被賦與固定數值的指令，被稱為常數。Forth 定義整數常數的方法如下：
```
<整數> constant <常數名>
<整數> <整數> 2constant <常數名>
<浮點數> fconstant <常數名>
```
指令 `constant` 從堆疊上取得整數，從輸入緩衝區取得跟在它後面的名稱，然後在字典裡建立了一個同名的常數。指令 `fconstant` 的行為類似，但定義的是浮點數常數。而 `2constant` 則從堆疊上取得兩個整數，為它們定義了一個常數。

例一：定義自己的真、假、圓週率常數
```
rf> -1 constant my-true
 ok
rf> 0 constant my-false
 ok
rf> 3.141592653e fconstant my-pi
 ok
rf> my-true .  my-false .  my-pi f.
-1 0 3.1415927  ok
```

例二：某機器只能大於或等於 4&deg;C ，小於 40&deg;C 時工作，超過就要停機，請將 4 和 40 這組數字定為一常數 `range` 並以 `within` 分別判斷 3 度、20 度及 40 度是否落在機器可以工作的範圍內。

```
4 40 2constant range
: in-range ( n -- )   range within ;
```

以下是使用 `constant` 和 `fconstant` 定義出來的指令在字典中的示意圖。

```
-1 constant my-true

            +---------+
        名稱 | my-true |
            +---------+       +---------------------------+
        行為 | const   | ----> | 將資料空間內的整數放上資料堆疊 | 程式碼空間
            +---------+       +---------------------------+
        資料 |         | --+
            +---------+   |   +----+
                          +-> | -1 | 資料空間
                              +----+

3.141592653e fconstant my-pi

            +--------------+
        名稱 | my-pi        |
            +--------------+      +-----------------------------+
        行為 | fconst       | ---> | 將資料空間內的浮點數放上浮點堆疊 | 程式碼空間
            +--------------+      +-----------------------------+
        資料 |              | --+
            +--------------+   |   +--------------+
                               +-> | 3.141592653e | 資料空間
                                   +--------------+

```

### 本節指令集
| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `constant <name>` | ( n -- ) &emsp; 使用 n 來定義一個名為 `name` 的指令。當這指令執行時，數字 n 會被放上堆疊。| constant |
| `2constant <name>` | ( n1 n2 -- ) &emsp; 使用 n1 n2 來定義一個名為 `name` 的指令。當這指令執行時，數字 n1 和 n2 會被放上堆疊。 | two-constant |
| `fconstant <name>` | ( F: r -- ) &emsp; 使用浮點數 r 來定義一個名為 `name` 的指令。當這指令執行時，浮點數 r 會被放上浮點堆疊。| f-constant |

-----------
## 資料空間

我們已見到冒號定義及常數如何使用資料空間。當我們想要設計複雜的資料結構時，必須先瞭解資料空間的特性。以下是我們定義了 `my-false` 、 `my-pi` 、 `range` 及 `in-range` 之後資料空間的示意圖。

```
資料空間

    較低的記憶位址                                    here
    +------+------+------+------+------+------+------+-------------------------
    | 3.141592653e|  4   |  40  | range|within| exit |
    +------+------+------+------+------+------+------+-------------------------
     my-pi         range         in-range             未使用的記憶空間
     8 bytes       4 + 4 bytes
     1 cell        2 cells       3 cells
```

資料依從低到高的次序放進資料空間中。記憶體位址的單位是位元組 (byte) 。但是大多數的資料大小超過一個位元組。比如上圖中的整數佔了 4 位元，浮點數佔了 8 位元。Forth 系統習慣將整數所佔的位元組數目稱為一個單元 (cell)。

| Fort  h | 作業系統 | 資料堆疊上整數所佔位元組數 | 資料空間中整數所佔位元組數 | 浮點數所佔位元組數 | cell 的位元組數 |
|---------|---------|-----------------------|------------------------|-----------------|---------------|
| rtForth | 64 位元 Linux | 8  | 4 | 8 | 4 |
| rtForth | 32 位元 Linux | 4  | 4 | 8 | 4 |
| SwiftForth  | 64 位元 Linux | 4  | 4 | 8 | 4 |
| gforth  | 64 位元 Linux | 8  | 8 | 8 | 8 |

上表中 rtForth 在 64 位元的作業系統下，資料堆疊的整數佔了 8 個位元組，也就是 64 位元。但資料空間中的整數只佔了 4 個位元組，也就是 32 位元。這使得放在資料堆疊上的整數可以比放在資料空間中的整數大得多。將資料堆疊上的整數放進資料空間中時，最高的 32 位元會被截去。如下例，
```
rf> $1234123412341234 constant all
 ok
rf> all h.
12341234  ok
```

由於 32 位元整數能表示的最大整數為 2147483647 。這範圍對工業控制綽綽有餘。此一現象不需要太過顧慮。
```
rf> $7fffffff .
2147483647  ok
```

Forth 指令 `cells` 可以得到一個單元所需的位元組數。
```
rf> 1 cells .
4  ok
```

指令 `floats` 則可以得到一個浮點數所需的位元組數。
```
rf> 1 floats .
8  ok
```

另外 Forth 還提供了指令 `cell+` 和 `float+`，可以將堆疊上的數字加上一個單元的位元組數或一個浮點數的位元組數。

```
rf> 1 cell+ .  1 float+ .
5 9  ok
```

### 一個單元的資料空間指令

Forth 指令 `here` 可以得到目前資料空間未使用部份的開始位址，也就是資料空間下一個可以使用的位置。我們先以 `-work  marker -work` 清除自己定義的指令，然後使用 `here .` 檢查一下下一個可以使用的資料空間的位址。然後再定義一個常數看看。
```
rf> -work  marker -work
 ok
rf> here h.
7F95210014E0  ok
rf> -1 constant my-true
 ok
rf> here h.
7F95210014E8  ok
```
因為新定義的指令使用了部份的資料空間。我們發現 `here` 的數值改變了。

你使用的 Forth 系統在清除了自己定義的指令後，`here` 的位置可能和本書例子中的不一致。嘗試以下例子時請依據自己系統的情形修改輸入的數值。

我們可以使用指令 `allot` 保留一部份資料空間給我們的程式使用。

```
rf> 1 cells allot here h.
7F95210014F0  ok
```

指令 `1 cells allot` 保留了一個單元，在此是 4 個位元組。因此 `here` 的值增加了。

指令 `!` 可以用來將一個整數存放在資料空間中某個記憶體位址內。而指令 `@` 可以將指定的記憶體位址內的整數取出放到堆疊上。
```
rf> 1234 $7F95210014E8 !
 ok
rf> $7F95210014E8 @ .
1234  ok
```
在上例中我們以指令 `!` 將 1234 放進剛剛保留了一個單元的記憶體位址 1260 中。然後再以指令 `@ .` 將位址 1260 內的整數印出來。果然那個位址內的整數是 1234。有一個指令 `?` 的行為就是 `@ .` 。因此我們也可以以下方式將 1260 內的資料印出。
```
rf> $7F95210014E8 ?
1234  ok
```

另有一指令 `dump` 可以讓我們檢視一塊記憶體的內容。
```
rf> $7F95210014E8 2 dump
7F95210014E8 :  D2  4  0  0   0  0  0  0 - 0  0  0  0   0  0  0  0  0_______________
 ok
rf> $4D2 .
1234  ok
```

指令 `+!` 會將指定位址內的整數加上堆疊上的數字。
```
rf> 1 $7F41CEE014E8 +!  $7F41CEE014E8 ?
1235  ok
```

指令 `,` 會配置一個單元的資料空間，然後把資料堆疊疊頂的整數放進這個剛配置的空間中。我們習慣將這種配置記憶體同時填入數值的行為稱為「編譯」。因為编譯指令 `if`， `."`， `;` 等都會做類似的事。我們會說指令 `,` 將一個整數「編譯」到字典裡面。
```
rf> here h.  1 ,  here h.  2 ,
7F41CEE014F0 7F41CEE014F8  ok
rf> $7F41CEE014F0 ?  $7F41CEE014F8 ?
1 2  ok
```
在上例中我們使用指令 `,` 將 1 、 2 編譯到字典裡。因為每個整數在資料空間中佔一個單元，我們可以知道每個整數所在的位址，並用指令 `?` 將它們印出來。

### 雙單元的資料空間指令

指令 `!` 和 `@` 存取一個單元的資料空間。Forth 也提供了 `2!` 和 `2@` 兩個指令用來存取兩個單元的資料空間。

請進行以下練習：

```
rf> -work marker -work
 ok
rf> here h.  2 cells allot
7F41CEE014E0  ok
rf> 1 2 $7F41CEE014E0  2!  .s
 ok
rf> $7F41CEE014E0 2@ .s
1 2  ok
rf> $7F41CEE014E0 dup @  swap cell+ @  .s
1 2 2 1  ok
rf> xx
xx Undefined word
```
我們先清除自行定義的指令，以 `allot` 配置兩個單元的資料空間。將整數 1、2 以 `2!` 放入配置的記憶體。以 `2@` 取出。期間我們使用 `.s` 來檢查堆疊是否如我們預期。我們還使用 `@` 依序取出這兩個單元的內容。我們發現堆疊上 ( 1 2 ) 這兩筆資料被放進字典的次序是先把疊頂的 2 放進低位址 1256 的記憶體內，再把底下的 1 放在高位址 1260 的記憶體內。最後我們執行文本直譯器不認得的指令 `xx` 以清除堆疊。

Forth 2012 標準並未提供相當於指令 `,` 的雙單元的編譯指令 `2,`。但 rtForth 和 gforth 都有提供。使用 SwiftForth 的人可以可以自行定義一個如下：
```
\ 將 n1 n2 編進字典裡
: 2, ( n1 n2 -- )   here  2 cells allot  2! ;
```

### 對齊

資料放進記憶體的開始位址應符合 CPU 的對齊 (alignment) 規則。比如 32 位元的整數因為是由 4 個位元組構成，應該放在開始位址是 4 的倍數的地方。而 64 位元的浮點數則應放在開始位址是 8 的倍數的地方。這方便 CPU 的資料匯流排 (data bus) 一次存取所有的資料。如果資料沒對齊它該有的位址，因為無法一次存取，指令執行的性能變差。甚至在某些 CPU 會無法正確取得資料。在 ARMv7 以上的版本以及 Intel 的 CPU 上，資料都能正確存取。就是性能需要注意。

| Forth   | 作業系統 | 整數對齊位址 | 浮點數對齊位址 |
|---------|---------|------------|-------------|
| rtForth | 64 位元 Linux | 4 的倍數 | 8 的倍數 |
| rtForth | 32 位元 Linux | 4 的倍數 | 8 的倍數 |
| SwiftForth | 64 位元 Linux | 4 的倍數 | 8 的倍數 |
| gforth | 64 位元 Linux | 8 的倍數 | 8 的倍數 |

指令 `aligned` 及 `faligned` 分別調整疊頂的整數使其符合整數及浮點的對齊原則。

例子：
```
rf> 0 aligned .  1 aligned .
0 8  ok
rf> 0 faligned .  1 faligned .
0 8  ok
```
上例中因為 1 不符合對齊原則而被 `aligned` 及 `faligned` 調整為符合的數值。

指令 `align` 和 `falign` 則分別調整 `here` 的回傳值使其符合整數及浮點數的對齊原則。

例子：
```
rf> 1 allot  here h.
7F41CEE014F1  ok
rf> align here h.
7F41CEE014F8  ok
rf> 2 allot here h.  falign here h.
7F41CEE014FA 7F41CEE01500  ok
```
上例中在執行了 `1 allot` 後，`here` 變成 $7F41CEE014F1，不符合對齊原則。我們以 `align` 修正使得 here 變成符合整數對齊原則的 $7F41CEE014F8，在 2 allot 之後，又以 `falign` 修正使其變成符合浮點數對齊原則的 $7F41CEE01500。

### 浮點數的資料空間指令

類似 `!` 、 `@` 和 `,`，Forth 提供了浮點數的版本： `f!` 、`f@` 、 `f,` 。
因為 `f,` 不是 Forth 2012 標準，如果你的系統沒有 `f,`，可以定義 `f,` 如下：
```
: f, ( F: r -- )   here  1 floats allot  f! ;
```
指令 `f,` 先以 `here` 取得下一個可用的資料空間，然後使用 `1 floats allot` 配置一個浮點數大小的空間，然後以指令 `f!` 把浮點堆疊上的浮點數 `r` 存在這個空間裡。

注意 `f,` 並未事先執行 `falign` 對齊浮點數所需位址。若重視存取的性能，應先以 falign 對齊。

請進行以下練習：

```
rf> -work marker -work
 ok
rf> here h.
7FEFB0E014E0  ok
rf> 2 ,  falign  1e f,  2e f,
 ok
rf> $7FEFB0E014E0  dup ?  cell+ faligned dup f@ f.  float+ dup f@ f.
2 1.0000000 2.0000000  ok
rf> $7FEFB0E014E0  20 dump
7FEFB0E014E0 :   2  0  0  0   0  0  0  0 -  0  0  0  0   0  0 F0 3F  ______________p?
7FEFB0E014F0 :   0  0  0  0   0  0  0 40 -  0  0  0  0   0  0  0  0  _______@________
 ok
```

在練習中，我們編譯了一個大小為二的浮點數陣列到字典中。首先我們使用 `3 ,` 先編譯了這個陣列的大小，也就是 3。因為這個編譯行為會造成 `here` 不再對齊浮點數所需的位址。所以我們使用 `falign` 調整 `here`，之後再以 `1e f, 2e f,` 將兩個浮點數編譯到字典中。

要取出那陣列的資料時，我們先以 `?` 印出陣列的長度，再以 `cell+ faligned` 跳過陣列的長度到下一個對齊浮點數的位址，以 `f@ f.` 印出，再以 `float+ dup f@ f.` 跳過這個浮點數並印出下一個。

在最後出於好奇我們以 `dump` 印出之前存在資料空間中的資料，在此不對其內容進行解讀。

### 本節指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|------------------------------------|--------|
| `cells` | ( n1 -- n2 ) &emsp; 傳回 n1 個單元所佔的位元組數 n2 | cells |
| `cell+` | ( a1 -- a2 ) &emsp; 將位址 a1 加上一個單元所佔的位元組數，結果是 a2 | cell+ |
| `floats` | ( n1 -- n2 ) &emsp; 傳回 n1 個浮點數所佔的位元組數 n2 | floats |
| `float+` | ( -- ) &emsp; 將位址 a1 加上一個浮點數所佔的位元組數，結果是 a2| float+ |
| `here` | ( -- ) &emsp; 將資料空間的下一個未被使用的位址放上堆疊 | here |
| `allot` | ( n -- ) &emsp; 從未被使用的資料空間中配置一下塊 n 個位元組的空間，常在 create 之後使用 | allot |
| `!` | ( n addr -- ) &emsp; 將 n 存在資料空間位址 addr 內 | store |
| `@` | ( addr -- n ) &emsp; 從資料空間位址 addr 取出一個單元大小的整數放上堆疊 | fetch |
| `?` | ( addr -- ) &emsp; 印出資料空間位址 addr 內一個單元大小的整數 | question |
| `+!` | ( n addr -- ) &emsp; 將位址 addr 內的整數加 n | plus-store |
| `,` | ( n -- ) &emsp; 從未被使用的資料空間中配置下一塊大小為一個單元的空間，並將 n 放進這個空間 | comma |
| `2@` | ( addr -- n1 n2 ) &emsp; 從資料空間位址 addr 處拿出兩個整數放上堆疊 | two-fetch |
| `2!` | ( n1 n2 addr -- ) &emsp; 將 n1 和 n2 放進資料空間位址 addr 處 | two-store |
| `2,` | ( n1 n2 -- ) &emsp; 從未被使用的資料空間中配置下一塊大小為兩個單元的空間，並將 n1 和 n2 放進這個空間 | two-comma |
| `f!` | ( addr -- ) ( F: r -- ) &emsp; 將 r 放進資料空間位址 addr 處  | f-store |
| `f@` | ( addr -- ) ( F: -- r ) &emsp; 從資料空間位址 addr 處拿出一個浮點數放上浮點堆疊 | f-fetch |
| `f,` | ( F: r -- ) &emsp; 從未被使用的資料空間中配置下一塊大小能容一個浮點數的空間，並將 r 放進這個空間 | f-comma |
| `align` | ( -- ) &emsp; 如果資料空間下一未被使用的位址不符合單元的對齊原則，修正使其對齊 | align |
| `aligned` | ( addr -- addr' ) &emsp; 如果資料空間的位址 addr 不符合單元的對齊原則，修正使其對齊 | aligned |
| `falign` | ( -- ) &emsp; 如果資料空間下一未被使用的位址不符合浮點數的對齊原則，修正使其對齊 | f-align |
| `faligned` | ( addr -- addr' ) &emsp; 如果資料空間的位址 addr 不符合浮點數的對齊原則，修正使其對齊 | f-aligned |
| `dump` | ( addr n -- ) &emsp; 以 16 進制及位元方式印出資料空間位址 `addr` 開始處 `n` 個字元的資料 | dump |

------
## 變數

如果我們想為我們配置好的記憶體取個名字，可以使用 constant。例如：

```
rf> here  1 ,  constant the-one
 ok
rf> the-one ?
1  ok
```

在上面，我們先將下個可以配置的資料空間的位址放上堆疊。然後，使用 `1 ,` 配置一個單元的空間並將數字 1 放進去，然後執行 `constant the-one`，指令 `constant` 取得保存在堆疊上的資料空間位址，也就是存放數字 1 的資料空間位址，將它命名為常數 `the-one`。當執行常數 `the-one` 時，對應的資料空間位址被放上堆疊。之後的指令 `?` 取出這個位址內的資料並印出。

因為常有將配置的記憶體命名的需求，Forth 提供了變數定義指令`variable` 、 `2variable` 及 `fvariable`。這幾個定義指令都會在配置記憶體時先以 `align` 或 `falign` 對齊好記憶體。執行由這些指令定義出來的變數時，會將配置的記憶體的開始位址放在堆疊上。可以使用上一章節提及的指令來存取這些記憶體。

例子：
```
rf> variable x 2variable xy fvariable fxy
 ok
rf> x h.  xy h.  fxy h.  here h.
7F6303C014F0 7F6303C014F8 7F6303C01508 7F6303C01510  ok
rf> x @ .  xy 2@ . .  fxy f@ f.
0 0 0 0.0000000  ok
rf> 1 x !  2 3 xy 2!  4e fxy f!
 ok
rf> x @ .  xy 2@ . .  fxy f@ f.
1 3 2 4.0000000  ok
```
在這例子中，新建立的變數的內容是 0 或是 0e。rtForth 會在建立變數時填入這些預設值。在 Forth 2012 標準中並未要求在建立時使用 0 或 0e 為預設值。因此使用其他 Forth 版本注意要給予正確的預設值。

通常 Forth 的程式在計算過程中會將結果放上堆疊，再以其他指令處理這些結果。有時不想立刻使用這些結果時，可以把結果放在變數中，需要時才拿出來使用。

例子：之前的章節提到了亂數產生器。那時我們將亂數的種子放在堆疊上。現在讓我們將種子放在變數裡。

```
: xorshift ( n -- x ) dup 13 lshift xor dup 17 rshift xor dup 5 lshift xor ;
variable seed   2463534242 seed !
: rnd ( n1 -- n2 )   seed @  xorshift  dup seed !  swap mod abs ;
```
以上的 `xorshift` 就是之前的 `xorshift`。我們以 `variable` 定義了一個變數 `seed`，並使用指令 `!` 隨便存了一個整數 2463534242 到這個變數中。之後定了一個指令 `rnd` 使用指令 `@` 從 `seed` 拿出目前的種子，使用 `xorshift` 算出新的種子，複製一份保存在 `seed` 中。然後把這新的種子依前一章的方式求 `n1` 的餘數，並以 `abs` 求正數。

測試一下：

```
rf> 100 rnd .
79  ok
rf> 100 rnd .
60  ok
rf> 100 rnd .
0  ok
```

### 本節指令集
| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `variable <name>` | ( -- ) &emsp; 定義一個名為 `name` 的指令，並配給它一個單元的資料空間。當執行這個指令時，空間的開始位址會被推上堆疊 | variable |
| `2variable <name>` | ( -- ) &emsp; 定義一個名為 `name` 的指令，並配給它兩個單元的資料空間。當執行這個指令時，空間的開始位址會被推上堆疊 | two-variable |
| `fvariable <name>` | ( -- ) &emsp; 定義一個名為 `name` 的指令，並配給它可容一個浮點數的資料空間。當執行這個指令時，空間的開始位址會被推上堆疊 | f-variable |

----------
## 向量執行

先看一段處理語言的程式。
```
0 constant english
1 constant italian
variable language   english language !
: greet   language @
    case
      english of ." Hello!" endof
      italian of ." Ciao!" endof
      ." Unknown language"
    endcase
;
```

執行一下，
```
rf> greet
Hello! ok
rf> italian language !
 ok
rf> greet
Ciao! ok
```

這程式雖然符合我們的預期，但有個缺點：每增加一種語言我們就需要修改 `greet` 這個指令。如果這是我們提供給別人的函ft式庫，那麼別人必須修改我們提供的指令。

一個解決的方法是使用「令牌」(execution token)。令牌是一個代表 Forth 指令的數字。我們可以使用這個數字來執行對應的指令。

先定義以下兩個指令，
```
: hello   ." Hello!" ;
: ciao    ." Ciao!" ;
```

我們可以使用指令 `'` 得到 `hello` 或 `ciao` 的執行令牌，再以 `execute` 執行它們。
```
rf> ' hello .   ' ciao .
219 220  ok
rf> ' hello execute
Hello! ok
rf> ' ciao execute
Ciao! ok
```

我們可以將不同語言問候指令的令牌放進變數中，
```
variable 'greet
: greet   'greet @ execute ;
```
在這兒我們遵照 Forth 的習慣：以 `'` 開始的名稱代表用來存放令牌的變數。
指令 `greet` 會從變數 `'greet` 中取出令牌來執行。因此我們只要改變變數 `'greet` 的內容，就可以改變指令 `greet` 的行為。

測試一下：
```
rf> ' hello  'greet !
 ok
rf> greet
Hello! ok
rf> ' ciao  'greet !
 ok
rf> greet
Ciao! ok
```

然後我們可以定義語言切換的指令。
```
: english   ['] hello  'greet ! ;
: italian   ['] ciao  'greet ! ;
```
在這兒，我們使用指令 `'` 的編譯版本 `[']`。指令 `[']` 會取得其後指令的令牌，編譯進字典中，在執行 `english` 或 `italian` 時這令牌會被推上堆疊。

當某種行為有直譯和編譯兩種版本時，Forth 習慣在直譯的版本外加上`[`和`]`來命名編譯的版本。像之前提過的 `[char]`，它的直譯版本是 `char`。測試一下：
```
rf> char * emit
* ok
```

我們以以下兩個程式說明直譯版和编譯版的不同：
```
: ex1   [char] * emit ;
: ex2   char * emit ;
```
以下是編譯的結果：

```
+-----+----+------+------+
| lit | 42 | emit | exit | ex1 的資料空間
+-----+----+------+------+
+------+---+------+------+
| char | * | emit | exit | ex2 的資料空間
+------+---+------+------+
```
所謂的冒號定義其實就是把定義中的 Forth 指令的令牌編到字典裡。執行一個冒號定義指令時，Forth 的內層直譯器 (inner interpreter) 會取得定義內的令牌，順序執行。

在定義 `ex1` 的時候，指令 `[char]` 會執行，得到之後 `*` 的 ASCII 碼 42 後，編了 `lit` 的令牌和數字 42 進字典裡。當 `ex1` 執行到 `lit` 時，指令 `lit` 會將之後的 42 放上堆疊。

在定義 `ex2` 時，指令 `char` 被編譯到字典中，因此並沒有取得之後 `*` 的 ASCII 碼。於是之後的 `*` 被認為是乘法指令，被編進字典中。直到 `ex2` 執行時，`char` 才會執行，並且取得在 `ex2` 之後的字元的 ASCII 碼放上堆疊。測試一下：
```
rf> 2 ex2 1
b ok
```
我們先放了一個數字 2，才執行 `ex2`。`ex2` 會得到之後的字元 `1` 的 ASCII 碼 49，將它乘以 2，得到 98，再以 `emit` 顯示得到 `b`。

將令牌放進記憶體中，留待未來執行的技巧被稱為向量執行 (vectored execution)。
冒號定義本身就是向量執行的一個例子。在下一章中的異常處理是另一個例子。

### 本節指令集
| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `['] <name>` | ( -- ) &emsp; 在冒號定義中使用，是一個編譯指令。會在字典中找尋名為 `name` 的指令，將其令牌編進冒號定義內。當之後冒號定義執行時，會將此一令牌推上堆疊 | bracket-tick |
| `execute` | ( xt -- ) &emsp; 執行令牌 xt 對應的指令 | execute |
| `' <name>` | ( -- xt ) &emsp; 在字典中找尋名為 `name` 的指令，將其令牌推上堆疊 | tick |
| `char <name>` | ( -- ) &emsp; 解析在 `char` 之後的輸入文字 `name`，將其第一個字元的 ASCII 碼放上堆疊 | char |

-------------------
## 定義自己的資料結構

### 使用 CREATE 定義新的資料結構

之前我們學到了幾個定義指令： `variable`, `2variable` 和 `fvariable`。在 rtForth 中，他們都是使用冒號定義出來的：
```
: variable   create  0 , ;
: 2variable   create  0 , 0 , ;
: fvariable   create  0e f, ;
```

指令 `create` 是最基本的定義資料結構的指令。它可以定義一個新的指令，當這指令執行時就會回傳資料空間下一個可配置的記憶體位址，指令 `create` 不配置任何記憶體給這新的指令使用。我們可以用 `allot` 、 `,` 、 `f,` 等配置適當的記憶體給這個指令使用。比如指令 `variable`，就是先用 `create` 建立了一個能回傳資料空間下一個可配置記憶體位址的指令，然後用 `0 ,` 在之後附加了一個長度為一個單元，內容為 0 的記憶體。

例子：用 `create` 定義一個 3&times;3 的單位浮點數矩陣。
一個 3&times;3 的單位矩陣總共有 9 個元素，其中有三個元素為 1.0，其他為 0.0。
```
\ 定義一個名為 m 的 3x3 浮點矩陣
create m
    falign
    1e f, 0e f, 0e f,
    0e f, 1e f, 0e f,
    0e f, 0e f, 1e f,
\ 將 matrix 的第 row 列第 col 行的內容取出
: m@ ( row col matrix -- ) ( F: -- value )
    faligned -rot
    swap 3 * +  floats +  f@ ;
\ 將 value 放進 matrix 的第 row 列第 col 行
: m! ( row col matrix -- ) ( F: value -- )
    faligned -rot
    swap 3 * +  floats +  f! ;
\ 印出 matrix 的所有元素
: .m ( matrix -- )
    0
    begin
      dup 9 <
    while
      ( matrix index ) dup floats  2 pick + f@
      9 3 f.r
      ( matrix index ) 1+
      ( matrix index ) dup 3 = over 6 = or if cr then
    repeat ( matrix index ) 2drop ;
```

指令 `create` 只會將資料空間的可配置位址，也就是 `here` 會回傳的位置，調整到對齊單元的位置。透過指令 `falign` 可對齊到浮點數所需的位置。因此在存取時也必須使用 faligned 來計算出這個經調整過的位置。雖然在目前的 rtForth 中，這帶來的性能改進不大。但在未來最佳化的 rtForth 版本，這可能會對程式執行效率有很大影響。

測試一下：
```
rf> 0 0 m m@ f.
1.0000000  ok
rf> 1 1 m m@ f.
1.0000000  ok
rf> 2 2 m m@ f.
1.0000000  ok
rf> 0 1 m m@ f.
0.0000000  ok
rf> 3e 0 1 m m! 
 ok
rf> m .m
    1.000    3.000    0.000
    0.000    1.000    0.000
    0.000    0.000    1.000 ok
rf> .s
 ok
```

最後我們使用 `.s` 檢查一下設計沒有不小心漏了一些資料在堆疊上。

### 定義自己的定義指令

上面的例子中我們使用 `create` 直接定義了矩陣 `m`，現在設計我們的第一個定義指令 `matrix`：
```
: matrix   create 
    falign
    1e f, 0e f, 0e f,
    0e f, 1e f, 0e f,
    0e f, 0e f, 1e f, ;
```

用它來定義多個矩陣：
```
rf> matrix m1
 ok
rf> matrix m2
 ok
rf> m1 .m
    1.000    0.000    0.000
    0.000    1.000    0.000
    0.000    0.000    1.000 ok
rf> m2 .m
    1.000    0.000    0.000
    0.000    1.000    0.000
    0.000    0.000    1.000 ok
```

### 使用 DOES> 定義資料結構的行為

指令 `2variable` 可以由冒號定義出來。但是指令 `2constant` 呢？指令 `2variable` 和 `2constant` 都會記住兩個整數。只是 `2variable` 回傳了存放整數資料的記憶體位址，而 `2constant` 會將這兩個數字從記憶體中拿出來放上堆疊。因此 `2constant` 和 `2variable` 只差在行為不同。以下是 `2constant` 的定義。

```
: 2constant ( n1 n2 -- )
    create  , ,              \ 定義時的行為
    does> ( -- n1 n2 )   2@  \ 被定義出來的指令執行時的行為
;
```
下圖是 `2constant` 本身以及被它定義出來的指令在字典中的示意圖。
```
: 2constant   create , , does> 2@ ;
4 40 2constant range

2constant
+--------+---+---+-------+------+----+------+
| create | , | , | _does | exit | 2@ | exit |
+--------+---+---+-------+------+----+------+
                                  ^
range                             |
                                  |
  action                          |
  +------+                        |
  | does |                        |
  +------+                        |
                                  |
  +---+                           |
  | x |---------------------------+
  +---+
  程式碼空間

  +---+----+
  | 4 | 40 |
  +---+----+
  資料空間
```

當 `4 40 2constant range` 執行時，`2constant` 會使用 `create , ,` 建造一個名為 `range`，資料空間內容為 4、40 的指令。然後執行由編譯指令 `does>` 編進字典的 `_does exit`，`_does` 修改被定義出來的 `range` 的行為，也就是圖中標示 action 的欄位，使其指向另一個函式 `does`。同時修改的還有程式碼空間中的欄位 ( 圖中的 x ) 使其指向 `does>` 後的 `2@ exit`。`_does` 之後的 `exit` 結束了 `2constant` 的執行。

當 `range` 執行時，因為它的行為是 `does`，會把資料空間的起始位址放上堆疊。然後依據程式碼空間中的 x 的指示，跳到 `does>` 之後的 `2@ exit` 那兒開始執行。

### 使用 +FIELD 定義欄位

之前定義矩陣時，我們並未定義距陣的各個欄位。這在 Forth 是常見的作法。但有的時候我們還是希望能為各個欄位取個容易記憶的名稱。Forth 2012 標準中提到了一個指令 `+field` 可以滿足我們的期望。它也是一個可以使用 `create ... does> ... ;` 定義出來的指令。

```
\ 定義時建立一個欄位，記住欄位在資料結構中的偏移量 offset，然後計算出下一個偏移量 offset' = offset + size，留給之後的指令使用。
\ 被定義出來的指令執行時，會將資料結構的開始位址 addr 加上之前記住的偏移量 offset，得到資料結構中所在欄位的位址 addr' = addr + offset。
: +field ( offset size -- offset' )
    create over , +
    does> ( addr -- addr' )   @ + ;
```

以下是 `+field` 的用法。
```
0                               \ 第一個欄位的位元組偏移量
  <位元組數一> +field <欄位名稱一> \ 定義第一個欄位
  <位元組數二> +field <欄位名稱二> \ 定義第二個欄位
constant <資料結構的位元組數名稱>   \ 為資料結構的大小取個名字
```
注意當有必要是使用 `aligned` 或 `faligned` 來調整偏移量。

例子：定義一個很簡單的資料結構 `person`。結構 `person` 有兩個欄位，第一個欄位是這個人的年齡，是一個整數。第二個欄位是他的月薪，是一個浮點數。
```
0
   1 cells +field person.age
   faligned
   1 floats +field person.salary
constant /person

: person   create /person allot ;
: .person ( 'person -- )  ." age: " dup person.age @ .  ." salary: " person.salary f@ f. ;
```

測試一下：
```
rf> person John
  ok
rf> 23 John person.age !
  ok
rf> 35000e John person.salary f!
  ok
rf> John .person
 age: 23  salary: 35000.0000000   ok
```

例子：定義一個二維的點。
```
\ 定義二維點的各欄位
0                         \ 第一個欄位的位元組偏移量
   1 floats +field p.x    \ 欄位 p.x 佔了一個浮點數大小
   1 floats +field p.y    \ 欄位 p.y 佔了一個浮點數大小
constant /point           \ 點的位元組數

\ point2 <name> 定義一個二維的點。
: point2   create  falign  /point allot  does> faligned ;
\ 印出點 p 的內容。
: .point ( p -- )
    [char] ( emit  dup p.x f@ f.
    [char] , emit      p.y f@ f.  [char] ) emit ;
```

測試一下：
```
rf> point2 p1
  ok
rf> 1e p1 p.x f!
  ok
rf> 2e p1 p.y f!
  ok
rf> p1 .point
(1.0000000 ,2.0000000 )  ok
```

### 陣列

Forth 2012 標準中並未提供定義陣列的指令。因為 Forth 的使用者很容易依自己的需求定義自己的陣列定義指令。
初學 Forth 的人可以從以下的兩種定義方式開始。其中較簡單版本的陣列索引從 0 開始，不做索引範圍檢查，因此性能比較好但使用者要注意安全。較複雜版本的索引從 1 開始，會檢查索引範圍，範圍有問題時會結束程式執行。因為在下一章才會說明程式遇到異常狀況的處理方式，本節的例子使用不做範圍檢查的陣列定義指令。

不檢查範圍，索引從 0 開始的版本：
```
: array ( capacity -- )
    create cells allot
    does> ( n -- a ) swap cells + ;
```
檢查範圍，索引從 1 開始的版本：
```
: array ( capacity -- )
    create  dup , cells allot
    does> ( n -- a )
      tuck  @ over <  over 1 <  or
      if ." Index out of range, " abort then
      cells + ;
```

以下我們使用一個控制紅綠燈的例子來展示這個陣列指令，以及之前提及的向量執行。

首先我們使用整數代表紅黃綠三種燈號。以下我們在常數名稱後加上 `#` ，代表數字或識別碼。這是常見的 Forth 風格。如果 `#` 出現在名稱開頭，常代表總數或大小。
```
0 constant red#
1 constant green#
2 constant yellow#
```

再來我們希望紅燈停 80 秒，緣燈行 50 秒，黃燈閃 30 秒。許多 Forth 系統會提供一個名為 `ms` 的指令，用來等待指定的毫秒數。我們在此使用一個點 `.` 來模擬 10 秒鐘。
```
\ 等待 n 個 10 秒
: 10-seconds ( n -- )
    begin dup 0>
    while [char] . emit  1-
    repeat drop ;
\ 亮紅燈並等待 80 秒後，將下一個狀態 ( green# ) 留在堆疊上。
: red ( -- green# ) ." red"   8 10-seconds  green# ;
\ 亮綠燈並等待 50 秒後，將下一個狀態 ( yellow# ) 留在堆疊上。
: green  ( -- yellow# ) ." green" 5 10-seconds  yellow# ;
\ 閃黃燈並等待 30 秒後，將下一個狀態 ( red# ) 留在堆疊上。
: yellow  ( -- red# ) ." yellow"  3 10-seconds  red# ;
```
上面的指令 `red` 、 `green` 和 `yellow` 有兩個作用，
* 執行在對應狀態所需的行為。
* 決定下一個狀況。

在較複雜的系統，這兩個作用常使用兩個不同的指令實作。

再來我們使用先前提過的向量執行技巧，但這次我們使用陣列而非變數。先定一個向量執行的陣列，
```
3 array vector

' red     0 vector !
' green   1 vector !
' yellow  2 vector !
```
使用向量執行的好處時，當有必要時我們可以很容易的改變執行的行為。比如我們可以設計一個除錯版的 `red-debug`，再使用以下方式改變紅燈的行為：

```
: red-debug ... ; \ 定義一個除錯版的紅燈處理指令
' red-debug   0 vector !
```

然後，我們要從紅燈開始，依照規則切換燈號。在這兒我們不想使用一個無窮迴圈，以免程式尚未測試完整一執行就停不下來。所以我們使用一個計算次數的變數，在執行到指定的次數時就停止。
```
variable down-counter  0 down-counter !
\ 控制燈號，初始狀態為 state#，狀態切換 #count 次就停止。
\ 參數 state# 可以是 red# 、 green# 、 yellow# 中的一個。
: control ( state# #count -- )
    down-counter !
    begin
      down-counter @ 0>
    while
      ( state# )
      -1 down-counter +!
      vector  @ execute ( state# )
    repeat ( state# )  drop ;
```

測試一下：
```
f> red# 1 control
 red........  ok
rf> red# 2 control
 red........ green.....  ok
rf> red# 3 control
 red........ green..... yellow...  ok
rf> red# 4 control
 red........ green..... yellow... red........  ok
```

Forth 的一大好處就是它能即寫即測。使用 Forth 的工程師常能很快的用 Forth 指令檢查設備的狀態，快速找出設備的軟硬體問題。

### 本節指令集
| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `create <name>` | ( -- ) &emsp; 以資料空間中下一個未被使用的位址來建立一個名稱為 `name` 的指令，當這指令執行時會將這個位址放上堆疊，在指令 `create` 之後通常會使用 `allot` 或是 `,` 等配置更多的空間 | create |
| `does>` | ( -- ) &emsp; 使用 Forth 指令來定義某個以 `create` 建造的指令的行為。當這個被 `create` 建造出來的指令執行時，和這指令結合的那塊資料空間的位址會先被放上堆疊，然後才執行 `does>` 之後的指令。 | does |
| `+field <name>` | ( n1 n2 -- n3 ) &emsp; 定義一個名稱為 `name` 的欄位，此欄位的偏移量為 n1 個位元組，大小為 n2 個位元組。定義好欄位後會在堆疊上留下下一個欄位的偏移量， n3 = n1 + n2 | plus-field |

-------------
## 本章重點整理

* 字典 (dictionary)
* 變數 (variable)
* 常數 (constant)
* 單元 (cell)
* 資料空間 (data space)
* 程式碼空間 (code space)
* 對齊 (alignment)
* 標記指令 (marker)
* 執行令牌 (execution token)
* 內層直譯器 (inner interpreter)
* 向量執行 (vectored execution)
* 定義指令 (defining word)

-----------
## 本章指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|------------------------------------|--------|
| `words` | ( -- ) &emsp; 顯示字典中目前能使用的指令 | words |
| `s" <string>"` | ( -- ) &emsp;  編譯其後的字串直到下一個 "。 | s-quote |
| `type` | ( addr n -- ) &emsp; 印出資料空間位置 addr 處，長度為 n 的字串 | type |
| `marker <name>` | ( -- ) &emsp; 定義一個用來刪除一群指令，名為 `name` 的標記指令。執行此一指令時，會刪除自己以及其後定義的所有指令，並釋放所有這些指令配置的記憶體。 | marker |
| `constant <name>` | ( n -- ) &emsp; 使用 n 來定義一個名為 `name` 的指令。當這指令執行時，數字 n 會被放上堆疊。| constant |
| `2constant <name>` | ( n1 n2 -- ) &emsp; 使用 n1 n2 來定義一個名為 `name` 的指令。當這指令執行時，數字 n1 和 n2 會被放上堆疊。 | two-constant |
| `fconstant <name>` | ( F: r -- ) &emsp; 使用浮點數 r 來定義一個名為 `name` 的指令。當這指令執行時，浮點數 r 會被放上浮點堆疊。| f-constant |
| `cells` | ( n1 -- n2 ) &emsp; 傳回 n1 個單元所佔的位元組數 n2 | cells |
| `cell+` | ( a1 -- a2 ) &emsp; 將位址 a1 加上一個單元所佔的位元組數，結果是 a2 | cell+ |
| `floats` | ( n1 -- n2 ) &emsp; 傳回 n1 個浮點數所佔的位元組數 n2 | floats |
| `float+` | ( -- ) &emsp; 將位址 a1 加上一個浮點數所佔的位元組數，結果是 a2| float+ |
| `here` | ( -- ) &emsp; 將資料空間的下一個未被使用的位址放上堆疊 | here |
| `allot` | ( n -- ) &emsp; 從未被使用的資料空間中配置一下塊 n 個位元組的空間，常在 create 之後使用 | allot |
| `!` | ( n addr -- ) &emsp; 將 n 存在資料空間位址 addr 內 | store |
| `@` | ( addr -- n ) &emsp; 從資料空間位址 addr 取出一個單元大小的整數放上堆疊 | fetch |
| `?` | ( addr -- ) &emsp; 印出資料空間位址 addr 內一個單元大小的整數 | question |
| `+!` | ( n addr -- ) &emsp; 將位址 addr 內的整數加 n | plus-store |
| `,` | ( n -- ) &emsp; 從未被使用的資料空間中配置下一塊大小為一個單元的空間，並將 n 放進這個空間 | comma |
| `2@` | ( addr -- n1 n2 ) &emsp; 從資料空間位址 addr 處拿出兩個整數放上堆疊 | two-fetch |
| `2!` | ( n1 n2 addr -- ) &emsp; 將 n1 和 n2 放進資料空間位址 addr 處 | two-store |
| `2,` | ( n1 n2 -- ) &emsp; 從未被使用的資料空間中配置下一塊大小為兩個單元的空間，並將 n1 和 n2 放進這個空間 | two-comma |
| `f!` | ( addr -- ) ( F: r -- ) &emsp; 將 r 放進資料空間位址 addr 處  | f-store |
| `f@` | ( addr -- ) ( F: -- r ) &emsp; 從資料空間位址 addr 處拿出一個浮點數放上浮點堆疊 | f-fetch |
| `f,` | ( F: r -- ) &emsp; 從未被使用的資料空間中配置下一塊大小能容一個浮點數的空間，並將 r 放進這個空間 | f-comma |
| `align` | ( -- ) &emsp; 如果資料空間下一未被使用的位址不符合單元的對齊原則，修正使其對齊 | align |
| `aligned` | ( addr -- addr' ) &emsp; 如果資料空間的位址 addr 不符合單元的對齊原則，修正使其對齊 | aligned |
| `falign` | ( -- ) &emsp; 如果資料空間下一未被使用的位址不符合浮點數的對齊原則，修正使其對齊 | f-align |
| `faligned` | ( addr -- addr' ) &emsp; 如果資料空間的位址 addr 不符合浮點數的對齊原則，修正使其對齊 | f-aligned |
| `variable <name>` | ( -- ) &emsp; 定義一個名為 `name` 的指令，並配給它一個單元的資料空間。當執行這個指令時，空間的開始位址會被推上堆疊 | variable |
| `2variable <name>` | ( -- ) &emsp; 定義一個名為 `name` 的指令，並配給它兩個單元的資料空間。當執行這個指令時，空間的開始位址會被推上堆疊 | two-variable |
| `fvariable <name>` | ( -- ) &emsp; 定義一個名為 `name` 的指令，並配給它可容一個浮點數的資料空間。當執行這個指令時，空間的開始位址會被推上堆疊 | f-variable |
| `['] <name>` | ( -- ) &emsp; 在冒號定義中使用，是一個編譯指令。會在字典中找尋名為 `name` 的指令，將其令牌編進冒號定義內。當之後冒號定義執行時，會將此一令牌推上堆疊 | bracket-tick |
| `execute` | ( xt -- ) &emsp; 執行令牌 xt 對應的指令 | execute |
| `' <name>` | ( -- xt ) &emsp; 在字典中找尋名為 `name` 的指令，將其令牌推上堆疊 | tick |
| `char <name>` | ( -- ) &emsp; 解析在 `char` 之後的輸入文字 `name`，將其第一個字元的 ASCII 碼放上堆疊 | char |
| `create <name>` | ( -- ) &emsp; 以資料空間中下一個未被使用的位址來建立一個名稱為 `name` 的指令，當這指令執行時會將這個位址放上堆疊，在指令 `create` 之後通常會使用 `allot` 或是 `,` 等配置更多的空間 | create |
| `does>` | ( -- ) &emsp; 使用 Forth 指令來定義某個以 `create` 建造的指令的行為。當這個被 `create` 建造出來的指令執行時，和這指令結合的那塊資料空間的位址會先被放上堆疊，然後才執行 `does>` 之後的指令。 | does |
| `+field <name>` | ( n1 n2 -- n3 ) &emsp; 定義一個名稱為 `name` 的欄位，此欄位的偏移量為 n1 個位元組，大小為 n2 個位元組。定義好欄位後會在堆疊上留下下一個欄位的偏移量， n3 = n1 + n2 | plus-field |
| `dump` | ( addr n -- ) &emsp; 以 16 進制及位元方式印出資料空間位址 `addr` 開始處 `n` 個字元的資料 | dump |
| `.memory` | ( -- ) &emsp; 印出資料空間和程式碼空間的使用情況。 | dot-memory |
