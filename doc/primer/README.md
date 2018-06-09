# rtForth 入門 

Forth 是種很適用於工業控制的語言。而 rtForth 是動程科技針對自家的軸控系統設計的 Forth 方言。為回饋 Forth 社群，rtForth 自始就是開源的。

rtForth 的 rt 有兩個意思。首先是 real-time 的意思。動程科技的軸控系統需要一款能在實時作業系統中執行的腳本語言。在實時環境下不允許動態配置記憶體，因此在開源社群中常用的語言如 Python、Lua 等都不適用。Forth 是唯一的選擇。

其次，rt 也代表了 Rust，。Rust 是 Mozilla 公司為了開發下一代安全且高性能的瀏覽器而設計的程式語言，具有安全 (Safety)、速度 (Speed)、并發 (Concurrency) 特性。目前已被國際大型軟體公司，包括 Docker、Facebook、Google 用於內部的關鍵技術中。其特性不僅適合用來開發安全、高性能的瀏覽器、伺服器，也適合用於軸控系統。

---------
## 本書內容

「rtForth 入門」透過例子展示 Foth 語言的語法和概念。前半部的 Forth 指令是 [FORTH 標準](https://forth-standard.org/standard/index) 指令集的子集，若有不屬於 [FORTH 標準](https://forth-standard.org/standard/index) 的指令，會特別說明。後半部進階的部份則是 rtForth 特有的內容，包括如何使用 Rust 擴充 rtForth 的指令集，以及如何使用動程科技的軸控系統。

某些其他 Forth 文件中常討論的主題，在本書中特意不討論，原因如下：

* Metacompiler 或 cross compiler：由於 rtForth 是以 Rust 語言寫成的，可以使用 Rust 本身的 cross compiler 將 rtForth  移植到不同的系統上。而且，rtForth 的設計重點是能作為一個 Rust 函式庫，整合進其他的 Rust 程式中，因此 rtForth 不自帶 Metacompiler 或 cross compiler。
* 正整數、雙整數及混合整數型別的指令： rtForth 專注於 32 位元以上的系統，整數的範圍足夠大，因此並未提供傳統 Forth 一定提供的正整數、雙整數、混合整數計算指令。這也是 rtForth 不符合 Forth 2012 標準的主要地方。
* 組合語言指令：對於需要性能的程式，可以使用 Rust 實現。或在 Rust 函式中使用 inline asm。因此 rtForth 不提供組合語言指令。
* 返回堆疊和 `R>` `R<` 等指令：`R>` 和 `R<` 以及返回堆疊是危險的，容易造成當機的指令。雖然 rtForth 目前提供 `R>` 和 `R<` 指令。但在未來會以 LOCAL 取代。這選擇也和 Rust 語言注意安全的哲學更相容。

「rtForth 入門」的撰寫參考了以下文獻，謹在此表達感謝，
* J.V. Noble 的 [A Beginner's Guide to Forth](http://galileo.phys.virginia.edu/classes/551.jvn.fall01/primer.htm)
* Forth Inc. Leo Brodie 的 [Starting Forth](https://www.forth.com/starting-forth/)
* [FORTH 標準](https://forth-standard.org/standard/index)

-------------
## 如何閱讀本書

建議依序閱讀各章節中進階課題以前的部份，並實際練習書中的例子。進階課題內的各章是獨立的，可以只在有興趣或需要時才閱讀。
現在就讓我們開始！

## 目錄

* [安裝 rtForth](installation.md)
* [Forth 計算機](calculator.md)
  * [整數計算](integer.md)
  * [浮點計算](float.md)
  * [WIP: 邏輯計算](logic.md)
* [進階課題](advanced.md)
  * [TODO: 矩陣運算](matrix.md)
  * [TODO: 範例：電子凸輪](cam.md)