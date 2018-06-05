# rtForth 入門 

FORTH 是種很適用於工業控制的語言。而 rtForth 是動程科技針對自家的軸控系統設計的 FORTH 方言。為回饋 FORTH 社群，rtForth 自始就是開源的。

rtForth 的 rt 有兩個意思。首先是 real-time 的意思。動程科技的軸控系統需要一款能在實時作業系統中執行的腳本語言。在實時環境下不允許動態配置記憶體，因此在開源社群中常用的語言如 Python、Lua 等都不適用。FORTH 是唯一的選擇。

其次，rt 也代表了 rust，。Rust 是 Mozilla 公司為了開發下一代安全且高性能的瀏覽器而設計的程式語言，具有安全 (Safety)、速度 (Speed)、并發 (Concurrency) 特性。目前已被國際大型軟體公司，包括 Docker、Facebook、Google 用於內部的關鍵技術中。其特性不僅適合用來開發安全、高性能的瀏覽器、伺服器，也適合用於軸控系統。

「rtForth 入門」透過例子展示 FORTH 語言的語法和概念。在前半部的內容和 ANSI FORTH 相容。後半部進階的部份則是 rtForth 特有的內容，包括如何使用 Rust 擴充 rtForth 的指令集，以及如何使用動程科技的軸控系統。

「rtForth 入門」的撰寫參考了 J.V. Noble 的 [A Beginner's Guide to Forth](http://galileo.phys.virginia.edu/classes/551.jvn.fall01/primer.htm)，謹在此表達感謝。

現在就讓我們開始！

* [安裝 rtForth](installation.md)
* [使用 Forth 進行數值計算](numeric.md)
* [使用 Forth 進行邏輯計算](logic.md)
