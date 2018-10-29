# rtForth 入門 

Forth 是種很適用於工業控制的語言。而 rtForth 是動程科技針對自家的軸控系統設計的 Forth 方言。為回饋 Forth 社群，rtForth 自始就是開源的。

rtForth 的 rt 有兩個意思。首先是 real-time 的意思。動程科技的軸控系統需要一款能在實時作業系統中執行的腳本語言。在實時環境下不允許動態配置記憶體，因此在開源社群中常用的語言如 Python、Lua 等都不適用。Forth 是唯一的選擇。

其次，rt 也代表了 Rust，。Rust 是 Mozilla 公司為了開發下一代安全且高性能的瀏覽器而設計的程式語言，具有安全 (Safety)、速度 (Speed)、并發 (Concurrency) 特性。目前已被國際大型軟體公司，包括 Docker、Facebook、Google 用於內部的關鍵技術中。其特性不僅適合用來開發安全、高性能的瀏覽器、伺服器，也適合用於軸控系統。

---------
## 本書目的

本書主要作為動程科技對內對外教育訓練的教材。

---------
## 本書內容

本書的網路版位於，

* [https://mapacode.github.io/rtforth/](https://mapacode.github.io/rtforth/)


本書「rtForth 入門」透過例子展示 Foth 語言的語法和概念。在本書中使用的 Forth 指令絕大多數屬於 [FORTH 標準](https://forth-standard.org/standard/index) 指令集的子集，若有不屬於 [FORTH 標準](https://forth-standard.org/standard/index) 的指令，會特別說明。

某些其他 Forth 文件中常討論的主題，在本書中特意不討論，原因如下：

* Metacompiler 或 cross compiler：rtForth 以 Rust 語言寫成的，並使用 Rust 本身的 cross compiler 將 rtForth  移植到不同的的支援 rust 的系統。而且，rtForth 的設計重點是能作為一個 Rust 函式庫，整合進其他的 Rust 程式中，因此 rtForth 不自帶 Metacompiler 或 cross compiler。
* 正整數、雙整數及混合整數型別的指令： rtForth 專注於 32 位元以上的系統，整數的範圍足夠大，因此目前並未提供傳統 Forth 一定提供的正整數、雙整數、混合整數計算指令。這是 rtForth 不符合 Forth 2012 標準的主要地方。
* 組合語言指令：對於需要性能的程式，可以使用 Rust 實現。或在 Rust 函式中使用 inline asm。因此 rtForth 不提供組合語言指令。
* 自訂編譯指令：對大多數動程科技的使用者而言，內建的編譯指令已經足夠，因此不想在本書增加這方面的內容增加使用者學習的負擔。


「rtForth 入門」的撰寫參考了以下文獻，謹在此表達感謝。對以上內容有興趣的讀者，可以在理解本書的內容後參考以下文獻。

* J.V. Noble 的 [A Beginner's Guide to Forth](http://galileo.phys.virginia.edu/classes/551.jvn.fall01/primer.htm)
* J.L. Bezemer 的 [And so forth...](https://thebeez.home.xs4all.nl/ForthPrimer/Forth_primer.html)
* Forth Inc. Leo Brodie 的 [Starting Forth](https://www.forth.com/starting-forth/)
* Forth Inc. Elizabeth D. Rather 的 [Forth Application Techniques](https://www.forth.com/forth-books/)
* Forth Inc. Edward K. Conklin 及 Elizabeth D. Rather 的  [Forth Programmer's Handbook](https://www.forth.com/forth-books/)
* Leonard Morgenstern 的 [Len's Forth Tutorial](http://www.forth.org/svfig/Len/Tutorils.htm)
* [FORTH 標準](https://forth-standard.org/standard/index)

-------------
## 如何閱讀本書

建議依序閱讀各章節中進階課題以前的部份，並實際練習書中的例子。進階課題內的各章是獨立的，可以只在有興趣或需要時才閱讀。
現在就讓我們開始！

## 目錄

* [簡介](README.md)
* [安裝 rtForth](installation.md)
* [Forth 計算機](calculator.md)
  * [整數運算](integer.md)
  * [浮點運算](float.md)
  * [比較及邏輯運算](logic.md)
  * [自己定義運算指令](colon.md)
* [Forth 程式入門](programming.md)
  * [選擇](selection.md)
  * [循環](repetition.md)
  * [字典](dictionary.md)
  * [多工、異常處理與文本直譯器](tasking.md)
  * [程式碼風格](style.md)
