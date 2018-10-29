# 多工、異常處理與文本直譯器

在這個章節中，我們討論 rtForth 的多工指令、異常處理的方式以及文本直譯器。

Forth 2012 的標準並未規範多工的指令集，同時也未規範文本直譯器的設計細節。
因此本章中許多指令都不是 Forth 2012 的標準指令。當遇到 Forth 2012
標準指令時會特別註明。

-------------
## 多工

多工處理 (Multitasking) 指的是電腦同時執行多個程式的能力。在 rtForth 中指的則是使用一個 rtForth 虚擬機 (Virtual Machine, VM) 同時執行多個 Forth 程式的能力。

### 工作 (Task)

RtForth 採行協作式 (Cooperative multitasking) 循環制 (round-robin) 的多工。並且提供了五個工作 (Task)，因此可以同時執行五個程式。

協作式的意思是，每個工作執行的程式都需要適時的放棄自己的執行權利，使得其他工作能取得虚擬機的使用權。如果有某個工作因為設計不良或其他原因不釋放權利，就會導至其他工作停頓。在 rtForth 中這個放棄自己執行權利的指令是 `pause`。

每個工作有一個非零的識別碼。依 RtForth 目前的設計，識別碼從 1 開始。

RtForth 系統冷起動 (Cold start) 時取得執行權利的是工作一，這個工作又被稱為操作者 (Operator) 。指令 `operator` 會得到這個工作的識別碼 (目前的設計為 1)。每個工作都可以使用指令 `me` 取得目前工作的識別碼。

一個工作可以使用指令 `activate` 指派另一工作執行程式。被指派了的工作會自動進入運行中的狀態，或稱為「清醒 (awake)」的狀態。但不會立刻執行。要等到這工作取得虚擬機的使用權時才會執行。

指令 `suspend` 可以暫停另一工作的執行，指令 `resume`被暫停的工作繼續運作。指令 `halt` 則使得一工作放棄原來的工作，進入一個無窮的等待迴圈。

循環制 (round-robin) 的意思是，這些工作是依序取得虛擬機的執行權利的。當工作一釋放了權利後，下個運行中的工作取得權利，直到它放棄權利，下下個運行中的工作又取得權利，最後輪完一圈，執行權利又回到工作一。

每一個工作有自己的堆疊、自己的解碼指標、自己的直譯或編譯的狀態。這使得工作在執行堆疊運算時不被其他工作打擾。在 RtForth 的設計中，所有的工作共用一個字典，共用一個輸出緩衝區，這使得工作間能透過字典交換資料，同時將訊息輸出到同一個緩衝區中。

在 RtForth 內建的五個工作中，工作一有自己的輸入緩衝區，用來處理使用者的輸入，我們稱這能和使用者互動的工作為終端工作 (Terminal task)，其他四個工作沒有自己的輸入緩衝區，被稱為背景工作 (Background task)。

使用 rtForth 的 Rust 函式庫可以對工作進行和本章不同的規畫，比如多於五個的工作，或是像動程科技設計的軸控系統，有兩個終端工作，三個背景工作，同時每個工作都有自己的輸入及輸出緩衝區。

在本書中其他章節中的所有指令都不會執行 `pause`。在本章中會執行 `pause` 的指令都會說明。

### 指令 `ACTIVATE` 和 `PAUSE`

先看以下的程式範例，並請實際執行。

```forth
rf> variable counter
 ok
rf> : up   2 activate  begin pause 1 counter +! again ;
 ok
rf> : down   2 activate  begin pause -1 counter +! again ;
 ok
rf> : watch ( n -- )   0 ?do pause counter ? loop ;
 ok
rf> up
 ok
rf> 5 watch
0 1 2 3 4  ok
rf> 8 watch
5 6 7 8 9 10 11 12  ok
rf> down
 ok
rf> 7 watch
12 11 10 9 8 7 6  ok
```
以上範例先定義了一個變數 `counter`，因為所有的工作共用一套字典，所有的工作都可以存取這個變數。

指令 `up` 使用指令 `activate` 指派工作二執行在 `activate` 後的程式，也就是從 `begin` 開始，到 `again` 的這個無窮迴圈。這個迴圈不斷將 `counter` 的內容加一，並且每加一次都會使用 `pause` 交出虚擬機的使用權。注意執行 `up` 只指派了工作二的工作內容，程式執行到 `activate` 後就返回了。在 `activate` 後的那個無窮迴圈要等到工作二取得虚擬機使用權後才會被執行。

指令 `down` 行為類似 `up`，但是在迴圈中不斷的將 `counter` 的內容減一。

指令 `watch` 根據堆疊上的次數監控變數 `counter` 的變化。同樣的，每印一次 `counter` 的內容，就會使用 `pause` 交出虚擬機的使用權。

於是，一開始時我們執行指令 `up` 指派工作二要往上計數，此時擁有虚擬機執行權的是工作一，也就是操作者 (operator)。之後，當工作一執行 `5 watch` 時，`watch` pause 了五次，使得工作二有五次機會取得虚擬機的執行權，因此往上計數了五次，所以我們看到畫面上出現了 `0 1 2 3 4`。事實上 `counter` 在第五次時已經被計數到 5，這數值在之後的 `8 watch` 才被印出來。

最後我們使用 `down` 重新指派工作二進行倒數。並用 `7 watch` 檢視 `counter` 的變化。

本章中所有含有 `pause` 的迴圈都先 `pause` ，再做其他的事。這使得這類指令都至少會執行一次 `pause` ，把虚擬機使用權交給其他工作。

### 指令 `SUSPEND` 和 `RESUME`

延續上一小節的例子，我們可以使用 `suspend` 暫停工作二，再以 `resume` 讓工作二繼續執行。

```forth
rf> 2 suspend
 ok
rf> 6 watch
6 6 6 6 6 6  ok
rf> 2 resume 6 watch
5 4 3 2 1 0  ok
```

### 指令 `NOD` 、`STOP` 和 `HALT`

工作執行的必須以無窮迴圈結束或是以 `stop` 結束。指令 `nod` 是為此設計的一個無窮迴圈。以下是 `stop` 和 `nod` 的定義：

```forth
: nod   begin pause again ;
: stop   me suspend pause ;
```

指令 `stop` 使用了指令 `me` 取得執行 `stop` 的工作的識別碼，再使用 `suspend` 暫停了這個工作，然後執行 `pause` 將虚擬機的使用權交出去。

以下是一個例子：

```forth
variable counter
rf> : up-down  2 activate  5 0 do pause 1 counter +! loop stop 3 0 do pause -1 counter +! loop nod ;
 ok
rf> : watch 0 ?do pause counter ? loop ;
 ok
rf> up-down 10 watch
0 1 2 3 4 5 5 5 5 5  ok
rf> 2 resume 10 watch
5 4 3 2 2 2 2 2 2 2  ok
```

指令 `up-down` 先連續將 `counter` 上數 5 次，然後以 stop 暫停執行，被喚醒後將 `counter` 倒數 3 次，然後執行 `nod` 這個無窮等待的迴圈。

我們也可以使用指令 `halt` 使得另一個工作執行 `nod` 這個無窮等待的迴圈。以下是 `halt` 的定義：

```forth
: halt ( n -- )   activate ['] nod handler!  nod ;
```
指令 `halt` 設定工作的異常處理指令為 `nod`，同時指派工作進入無窮等待的迴圈。下一節會說明 `handler!` 這個指令以及異常處理程式。

有興趣的讀者可以試試讓工作執行到結尾的分號，比如
```forth
: wrong   2 activate ;
```
目前 rtForth (v0.5.0) 並未為針對這一行為進行保護。因此執行以上程式可能會因不同版本的 rtForth 而有不同的結果。大多數都會造成程式結束或是程式記憶區段錯誤。因此在工作中請務必以內有執行 `pause` 的無窮迴圈或 `nod` 或 `stop` 結束。而若是使用 `stop` 結束，請務必不要 `resume` 這個工作，導至最後仍執行到尾端的分號。

### 指令 `GET` 和 `RELEASE`

當多個工作共享一個系統資源，且使用這資源中途有可能以 `pause` 釋放虚擬機使用權時，必須考慮如果另一個工作搶用這個資源是否會造成問題。若有必要，可以使用一個變數記錄資源目前的使用者，當這變數內容為 0 時，代表沒有工作正在使用這資源，此時工作可以將自己的識別碼放進變數中，告知其他的工作這資源已經被佔用。當這變數內容為另一個工作的識別碼時，就必須等待那個工作釋放了這項資源，變數內容為零時才可以使用。

指令 `get` 和 `release` 就是在處理這種資源共享的情況。以下是 rtForth 中 `get` 和 `release` 的定義。

```forth
\ 取得資源 a，如果 a 正被其他工作佔用，則等待 a 被釋放後，在 a 中填入自己的識別碼。
: get ( a -- )   begin  pause dup @  while repeat me swap ! ;
\ 如果資源 a 中的識別碼是自己，那就填入 0 釋放這項資源。
: release ( a -- )   dup @ me = if 0 swap ! else drop then ;
```

以上程式中使用指令 `me` 取得目前工作的識別碼。

以印表機這項資源為例，當一工作正在列印時，另一工作需要等待。因此，

```forth
variable printer
: print   printer get  (print)  printer release ;
```
以上程式中的 `printer` 是一個代表某項資源的變數，而指令 `(print)` 執行實際的列印動作。在列印之前必須先以 `printer get` 取得印表機的使用權，並在列印完成後以 `printer release` 釋放這項資源。

要注意當有多於一個以上的系統資源時，每個工作最好一次只取得一項資源的使用權，用完釋放使用權後才再取得另一項資源的使用權。否則很有可能造成所謂的死結 (deadlock)，以下是一個死結的例子：

```forth
variable a
variable b
: start-task2   2 activate  a get b get  do-task2  a release b release nod ;
: start-task3   3 activate  b get a get  do-task3  b release a release nod ;
```
很可能因為工作二佔用了資源 a，工作三佔用了資源 b，彼此都等不到對方釋放資源。

### 時間

有時，一個工作在進行完某個步驟後，需要等待一段時間，才能進行另一個步驟。這時可以使用指令 `ms`。延續之前 `up` 那個範例，假設我們不想時時監控 `counter` 的值，只要每 100 毫秒監控一次就好。重新定義 `watch` 如下：

```forth
rf> : watch ( n -- )   0 ?do 100 ms counter ? loop ;
 ok
rf> up
 ok
rf> 10 watch
19457 41712 63963 86238 108512 130786 153056 175337 197533 219802  ok
```

如本章其他使用 pause 的指令，在這 `watch` 的迴圈中我們也先執行 `100 ms` 釋放虚擬機使用權，等取回使用權後才做其他的事。

以下是指令 `ms` 的定義，它使用指令 `mtime` 取得時間，之後不斷 `pause` 直到最新的時間和之前的時間差大於堆疊上的數字 `n` 為止。`ms` 和 `mtime` 使用的時間單位都是毫秒。

```forth
: ms ( n -- )   mtime  begin pause mtime over -  2 pick <  while repeat  2drop ;
```

另有一個類似 `mtime` 的指令 `utime`，但用來取得準確到微秒的時間。注意在 32 位元的系統上，`utime` 和 `mtime` 得到的時間是一個 32 位元的數字，能表達的最大時間有限。`mtime` 能表達約 24 天的時間。而 `utime` 只能表達到 35 分鐘。但它們很適合用來表示控制設備時的時間延遲，或是用來計算其他 Forth 指令執行所需的時間。

由於常有需要瞭解 Forth 指令執行的時間，rtForth 配合 `utime` 提供了三個指令 `xtime` 、`.xtime` 和 `0xtime` 來進行執行時間分析。說明如下，

* `xtime ( t0 xt -- )` 計算 `t0` 到現在的時間後，統計令牌 `xt` 對應的指令的最大及最小執行時間。
* `.xtime ( -- )` 印出所有指令的最大及最小執行時間。
* `0xtime ( -- )` 清除所有指令的最大及最小執行時間。

請見下例：

```
rf> utime ' noop xtime .xtime
noop|88,88 ok
rf> : noops   0 ?do utime ['] noop xtime loop ;
 ok
rf> 80 noops
 ok
rf> .xtime
noop|4,88 ok
rf> 0xtime
 ok
rf> .xtime
 ok
```

在例子中的 noop 是一個什麼也不做的 rtForth 指令。我們先使用 `utime` 取得目前的時間，再使用 `' noop` 取得 `noop` 的令牌。再以 `xtime` 執行這個令牌並分析執行時間。然後以 `.xtime` 印出統計結果。得到的 `noop|88,88` 顯示執行 `noop` 需要 88 微秒。這並不是正確的時間，因為這 88 微秒包括了直譯器從輸入緩衝區讀取指令並在在字典中找到 `noop` 然後以 `xtime` 執行的時間。

然後我們定義了 `noops`，讓我們可以執行多次 `noop` 並統計其時間。在執行了 `80 noops` 後，以 `.xtime` 印出統計結果，`noop` 的最小執行時間是 4 微秒，最大是剛剛的 88 微秒。因為在冒號和分號之間的指令是被編譯的，因此這最小時間 4 微秒不包括從輸入緩衝區讀取以及在字典中搜尋的時間，但依舊不是正確的時間。因為它還包括了 `['] noop` 將令牌放上堆疊，以及 `xtime` 從堆疊取出令牌並執行的時間。同時，因為 `utime` 最小能處理的時間是 1 微秒，小於 1 微秒的時間是無法量得的。

雖然如此，這幾個指令仍提供我們很有意義的資訊，有助於我們必要時改進 Forth 指令或是在實時系統下多工時的性能。

### 本節指令集

本節指令都非 Forth 2012 標準指令。指令集的設計參考了 Forth Inc. 的 SwiftOS 的多工指令集。
其中 `activate` 、`pause` 和 `ms` 常見於各種不同的 Forth 系統多工指令集。

| 指令 | 堆疊效果及指令說明                          | 口語唸法 |
|-----|------------------------------------------|--------|
| `me` | ( -- n ) &emsp; 目前工作的識別碼 | me |
| `activate` | ( n -- ) &emsp; 指派工作 `n` 的工作內容，並喚醒工作 `n`。 | activate |
| `pause` | ( n -- ) &emsp; 將虚擬機的使用權交給下一個醒著的工作。 | pause |
| `suspend` | ( n -- ) &emsp; 暫停工作 `n`，使它進入休眠狀態。 | suspend |
| `resume` | ( n -- ) &emsp; 恢復工作 `n` 的執行，使它進入清醒狀態。 | resume |
| `stop` | ( -- ) &emsp; 使目前的工作進入休眠狀態。 | stop |
| `nod` | ( -- ) &emsp; 一個不斷 `pause` 的無窮迴圈。 | nod |
| `halt` | ( n -- ) &emsp; 使工作 `n` 執行 `nod` 。| halt |
| `get` | ( n -- ) &emsp; 取得資源變數 `n` 的使用權。若變數 `n` 已被其他工作佔用，等待直到其他工作釋放此一變數。| get |
| `release` | ( n -- ) &emsp; 釋放資源變數 `n` 。| release |
| `mtime` | ( -- n ) &emsp; 目前的系統時間。單位為毫秒。| m-time |
| `ms` | ( n -- ) &emsp; 等待 `n` 毫秒。 | ms |
| `utime` | ( -- n ) &emsp; 目前的系統時間。單位為微秒。| u-time |
| `xtime` | ( t0 xt -- ) &emsp; 計算 `t0` 到現在的時間後，統計令牌 `xt` 對應的指令的最大及最小執行時間。`t0` 的單位是微秒。| x-time |
| `.xtime` | ( -- ) 印出所有指令的最大及最小執行時間。如果時間為 0 則不顯示。| dot-x-time |
| `0xtime` | ( -- ) 清除所有指令的最大及最小執行時間。 | zero-x-time |

-------------
## 異常處理

當我們執行一個字典中沒有的指令 `xx` 時，會印出 `Undefined` 的錯誤訊息。當執行 `0 0 /` 時，會印出 `Division by zero` 的鐄誤訊息。

```
rf> xx
xx Undefined word
rf> 0 0 /
/ Division by zero
```

當 RtForth 發現錯誤時，會先執行預設的異常處理指令。然後，重設系統，執行系統的文本直譯器。下列程式中的 `(abort)` 指令就是 RtForth 的異常處理程式。以下說明它使用到的幾個指令：

* `0stacks`：清空堆疊。
* `error`：取得錯誤碼。如果不為零代表有異常發生。
* `.token`：印出文本直譯器最近讀到的指令。
* `.error`：印出錯誤訊息。
* `0error`：清除錯誤碼。
* `flush-output`：印出輸出緩衝區的內容。
* `quit`： 重設工作，並執行工作的預設行為。通常這預設的行為就是 Forth 的文本直譯器。

```forth
: (abort)
    0stacks error -2 1 within not if
      .token space .error
    then flush-output 0error quit ;
```
指令 `(abort)` 只在錯誤碼不是 0、-1、-2 時才會印出錯誤訊息。0 代表沒有錯誤，-1 是由某個名為 `abort` 的指令發出的錯誤碼，系統收到這個錯誤碼時，只執行錯誤處理，不印出任何訊息。-2 是由另一個名為 `abort"` 的指令發出的錯誤訊息。目前 rtForth 尚未支援指令 `abort"`。

使用者的程式中可以使用指令 `abort` 放棄程式的執行，清除堆疊，回到工作的預設行為。

若有必要，我們可以修改異常處理指令，甚至修改系統冷起動後的預設行為。本節會說明如何修改異常處理指令，下一節說明 `quit` 以及如何修改預設行為。

在之前的章節曾提到向量執行。RtForth 在異常處理上使用向量執行的概念，每個工作都有自己的異常處理向量。我們可以使用指令 `handler!` 來修改這向量，以改變異常處理的行為。在上一節，指令 `halt` 使用 `['] nod handler!` 設定工作的異常處理指令為 `nod`。RtForth 的冷起動指令 `cold` 設定了所有工作預設的異常處理指令以及預設的行為，以下是它的定義：

```forth
\ Cold start
: cold
    2 halt  3 halt  4 halt  5 halt
    ['] (abort) handler!  quit ;
```

執行冷起動的工作是操作者，也就是工作一。首先它使用 `halt` 設定了工作 2-5 的異常處理程式為 `nod`，並指派它們的預設行為同樣是 `nod`。之後 `cold` 取得異常處理指令 `(abort)` 的執行令牌，並使用指令 `handler!` 設定它為工作一的異常處理指令。最後執行 `quit` 重設工作一，並啟動文本直譯器。

以下範例修改了工作一的異常處理向量，以不同的方式顯示錯誤訊息：

```forth
rf> : (my-abort)   0stacks error -2 1 within not if .token ."  ===> " .error 0error then flush-output quit ;
 ok
rf> ' (my-abort) handler!
 ok
rf> 0 0 /
/ ===> Division by zero
```

### 本節指令集

本節指令中僅 `quit` 和 `abort` 為 Forth 2012 的標準指令。

| 指令 | 堆疊效果及指令說明                          | 口語唸法 |
|-----|------------------------------------------|--------|
| `quit` | ( -- ) &emsp; 重設工作，並執行工作的預設行為。通常這預設的行為就是 Forth 的文本直譯器。| quit |
| `abort` | ( -- ) &emsp; 放棄程式的執行，清除堆疊，回到工作的預設行為。| abort |
| `handler!` | ( xt -- ) &emsp; 設定目前工作的異常處理指令為 `xt` 。| handler-store |
| `error` | ( -- n ) &emsp; 取得錯誤碼。如果不為零代表有異常發生。| error |
| `0error` | ( -- ) &emsp; 清除錯誤碼。| zero-error |
| `.error` | ( -- ) &emsp; 印出錯誤訊息。| dot-error |
| `.token` | ( -- ) &emsp; 印出文本直譯器最近讀到的指令。| dot-token |
| `0stacks` | ( -- ) &emsp; 清空資料堆疊和浮點堆疊 | zero-stacks |
| `flush-output` | ( -- ) &emsp; 印出輸出緩衝區的內容。| flush-output |
| `cold` | ( -- ) &emsp; 冷起動 rtForth。 | cold |

-------------
## 文本直譯器

上一節提到，冷起動時 rtForth 會設定各個工作的預設工作內容，並且設定它們的異常處理指令。因此，只要修改 `cold` 的定義，我們可以讓 rtForth 冷起動就執行我們自己的應用程式，比如控制一台設備，或是搜集外部感測器的資訊。

但現在先讓我們瞭解 rtForth 冷起動時的預設行為 `quit`，以下是它的程式。

```forth
\ 文本直譯器
: evaluate-input
    begin parse-word
      token-empty? not
    while
      compiling? if compile-token ?stacks else interpret-token ?stacks then
    repeat ;

\ Operator 工作的預設行為，重設返回堆疊後進入一個接受使用者輸入，
\ 並執行文本直譯器的無窮迴圈。若文本直譯器執行過程中有異常，會透
\ 過呼叫異常處理指令，重新執行 `quit`。
: quit
    reset
    begin receive evaluate-input
    ."  ok" flush-output
    again ;
```

指令 `evaluate-input` 就是所謂的文本直譯器，它使用了幾個我們之前未接觸過的指令，這些指令大多使用 rust 語言實作，在 rtForth 的開源程式中可以找到。指令的行為如下：

* `parse-word ( -- )`：從輸入緩衝區取出一串由空白字元 (空格和換行) 隔開的字串 (token)。
* `token-empty? ( -- t )`：取得的字串是否是空字串？空字串代表輸入緩衝區內所有的資料都被處理完了。
* `compiling? ( -- t )`：現在是否在編譯模式？編譯模式就是由冒號定義指令 : 開始，由分號指令 ; 結束的模式。
* `compile-token ( -- )`：判斷取得的字串是字典中的指令，還是整數，或是浮點數，並依據判斷結果將它編譯進字典中。如果編譯失敗，會呼叫異常處理指令。
* `interpret-token ( -- )`：判斷取得的字串是字典中的指令，還是整數，或是浮點數，並依據判斷結果執行它。如果執行失敗，會呼叫異常處理指令。
* `?stacks ( -- )`：檢查浮點堆疊或資料堆疊是否有滿溢 (overflow) 或不足 (underflow) 的錯誤。如果有就執行異常處理指令。

所以，`evaluate-input` 這個文本直譯器逐字的讀取工作的輸入緩衝區，如果緩衝區內已無資料，就以正常的方式回到 `quit`，否則依據工作的狀態 (編譯與否) 以及取得的字串的類型 (指令、整數、浮點數或是不認得的字串) 決定要編譯或執行這個字串，如果有編譯或執行有錯會呼叫上一節提到的異常處理程式直接回到 `quit`，在成功編譯或執行了這字串後，會檢查堆疊是否有異常，若有異常也會呼叫異常處理程式回到 `quit`。

指令 `quit` 兩個我們之前未討論過的指令，

* `reset ( -- )`：等待使用者輸入指令，並將輸入放進輸入緩衝區。

當 `quit` 成功執行 `evaluate-input` 後，會印出 ok 。但如果執行過程中發生異常，`quit` 會被重頭執行，也就不會印出 ok 。

感謝您花了不少時間閱讀這本手冊，也恭喜您，您學會了 rtForth。如果對本手冊或是 rtForth 有任何建議，請在以下網址提出您的 issues。

[https://github.com/mapacode/rtforth](https://github.com/mapacode/rtforth)

### 本節指令集

本節指令都非 Forth 2012 標準指令。

| 指令 | 堆疊效果及指令說明                          | 口語唸法 |
|-----|------------------------------------------|--------|
| `evaluate-input` | ( -- ) &emsp; 執行文本直譯器。| evaluate-input |
| `token-empty?` | ( -- t ) &emsp; 取得的字串是否是空字串？若是，`t` 為真。空字串代表輸入緩衝區內所有的資料都被處理完了。| token-empty |
| `compiling?` | ( -- t ) &emsp; 現在是否在編譯模式？若是，`t` 為真。編譯模式就是由冒號定義指令 : 開始，由分號指令 ; 結束的模式。| compiling |
| `compile-token` | ( -- ) &emsp; 判斷取得的字串是字典中的指令，還是整數，或是浮點數，並依據判斷結果將它編譯進字典中。如果編譯失敗，會呼叫異常處理指令。| compile-token |
| `interpret-token` | ( -- ) &emsp; 判斷取得的字串是字典中的指令，還是整數，或是浮點數，並依據判斷結果執行它。如果執行失敗，會呼叫異常處理指令。| interpret-token |
| `?stacks` | ( -- ) &emsp; 檢查浮點堆疊或資料堆疊是否有滿溢 (overflow) 或不足 (underflow) 的錯誤。如果有就執行異常處理指令。| question-stacks |
| `parse-word` | ( -- ) &emsp; 從輸入緩衝區取出一串由空白字元 (空格和換行) 隔開的字串 (token)。| parse-word |
| `reset` | ( -- ) &emsp; 重設返回堆疊，清除輸入緩衝區，進入非編譯模式，清除錯誤。| reset |
| `receive` | ( -- ) &emsp; 等待使用者輸入指令，並將輸入放進輸入緩衝區。| receive |

-------------
## 本章重點整理

* Terminal task：具輸入和輸出緩衝區，方便和使用者互動的工作。
* Background task：不具備輸入和輸出緩衝區，或不具備輸入緩衝區但和 Terminal task
  共用輸出緩衝區的工作。多用於設備的控制或長時間的運算。
* 異常處理指令：每個工作可以設定自己的異常處理指令。當工作中有錯誤發生時，會執行異常處理指令，並透過這指令重新開始工作的預設行為。

-------------------------------------
## 本章指令集

| 指令 | 堆疊效果及指令說明                          | 口語唸法 |
|-----|------------------------------------------|--------|
| `me` | ( -- n ) &emsp; 目前工作的識別碼 | me |
| `activate` | ( n -- ) &emsp; 指派工作 `n` 的工作內容，並喚醒工作 `n`。 | activate |
| `pause` | ( n -- ) &emsp; 將虚擬機的使用權交給下一個醒著的工作。 | pause |
| `suspend` | ( n -- ) &emsp; 暫停工作 `n`，使它進入休眠狀態。 | suspend |
| `resume` | ( n -- ) &emsp; 恢復工作 `n` 的執行，使它進入清醒狀態。 | resume |
| `stop` | ( -- ) &emsp; 使目前的工作進入休眠狀態。 | stop |
| `nod` | ( -- ) &emsp; 一個不斷 `pause` 的無窮迴圈。 | nod |
| `halt` | ( n -- ) &emsp; 使工作 `n` 執行 `nod` 。| halt |
| `get` | ( n -- ) &emsp; 取得資源變數 `n` 的使用權。若變數 `n` 已被其他工作佔用，等待直到其他工作釋放此一變數。| get |
| `release` | ( n -- ) &emsp; 釋放資源變數 `n` 。| release |
| `mtime` | ( -- n ) &emsp; 目前的系統時間。單位為毫秒。| m-time |
| `ms` | ( n -- ) &emsp; 等待 `n` 毫秒。 | ms |
| `utime` | ( -- n ) &emsp; 目前的系統時間。單位為微秒。| u-time |
| `xtime` | ( t0 xt -- ) &emsp; 計算 `t0` 到現在的時間後，統計令牌 `xt` 對應的指令的最大及最小執行時間。`t0` 的單位是微秒。| x-time |
| `.xtime` | ( -- ) 印出所有指令的最大及最小執行時間。如果時間為 0 則不顯示。| dot-x-time |
| `0xtime` | ( -- ) 清除所有指令的最大及最小執行時間。 | zero-x-time |
| `quit` | ( -- ) &emsp; 重設工作，並執行工作的預設行為。通常這預設的行為就是 Forth 的文本直譯器。| quit |
| `abort` | ( -- ) &emsp; 放棄程式的執行，清除堆疊，回到工作的預設行為。| abort |
| `handler!` | ( xt -- ) &emsp; 設定目前工作的異常處理指令為 `xt` 。| handler-store |
| `error` | ( -- n ) &emsp; 取得錯誤碼。如果不為零代表有異常發生。| error |
| `0error` | ( -- ) &emsp; 清除錯誤碼。| zero-error |
| `.error` | ( -- ) &emsp; 印出錯誤訊息。| dot-error |
| `.token` | ( -- ) &emsp; 印出文本直譯器最近讀到的指令。| dot-token |
| `0stacks` | ( -- ) &emsp; 清空資料堆疊和浮點堆疊 | zero-stacks |
| `flush-output` | ( -- ) &emsp; 印出輸出緩衝區的內容。| flush-output |
| `cold` | ( -- ) &emsp; 冷起動 rtForth。 | cold |
| `evaluate-input` | ( -- ) &emsp; 執行文本直譯器。| evaluate-input |
| `token-empty?` | ( -- ) &emsp; 取得的字串是否是空字串？若是，`t` 為真。空字串代表輸入緩衝區內所有的資料都被處理完了。| token-empty |
| `compiling?` | ( -- ) &emsp; 現在是否在編譯模式？若是，`t` 為真。編譯模式就是由冒號定義指令 : 開始，由分號指令 ; 結束的模式。| compiling |
| `compile-token` | ( -- ) &emsp; 判斷取得的字串是字典中的指令，還是整數，或是浮點數，並依據判斷結果將它編譯進字典中。如果編譯失敗，會呼叫異常處理指令。| compile-token |
| `interpret-token` | ( -- ) &emsp; 判斷取得的字串是字典中的指令，還是整數，或是浮點數，並依據判斷結果執行它。如果執行失敗，會呼叫異常處理指令。| interpret-token |
| `?stacks` | ( -- ) &emsp; 檢查浮點堆疊或資料堆疊是否有滿溢 (overflow) 或不足 (underflow) 的錯誤。如果有就執行異常處理指令。| question-stacks |
| `parse-word` | ( -- ) &emsp; 從輸入緩衝區取出一串由空白字元 (空格和換行) 隔開的字串 (token)。| parse-word |
| `reset` | ( -- ) &emsp; 重設返回堆疊，清除輸入緩衝區，進入非編譯模式，清除錯誤。| reset |
| `receive` | ( -- ) &emsp; 等待使用者輸入指令，並將輸入放進輸入緩衝區。| receive |
