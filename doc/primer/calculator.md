# 使用 Forth 計算

本節將 rtForth 當成計算機來用。

在前一章節中編譯產生的 ./target/release/rf 或是 ./target/debug/rf 只包括了 rtForth 的基本指令集，本節需要更多的指令，因此請以以下方式執行 rtForth：

執行除錯版的 rtForth： 

```
$ ./target/debug/rf lib.fs
```

執行最佳化版本的 rtForth： 

```
$ ./target/release/rf lib.fs
```

在 `rf` 命令後接上 `lib.fs` 會載入 lib.fs 內的指令集。檔案 `lib.fs` 內以 Forth 語言定義了常見的 Forth 指令。有興趣的人可以參考。使用者自行定義的指令集也可以以類似方式載入。假設使用者將自行開發的指令集放在 `path/to/user.fs` 中，而此指令集會使用到 `lib.fs` 內的指令，則可以以下方式執行 `rf` 以載入指令：

```
$ ./target/release/rf lib.fs path/to/user.fs
```

有載入指令集的情形下，rf 不會印出以下訊息：

```
rtForth v0.3.0, Copyright (C) 2018 Mapacode Inc.
Type 'bye' or press Ctrl-D to exit.
```

這是考慮到也許使用者想顯示自訂的顯示訊息。

## 整數運算

在 `rf>` 提示後輸入 `2 17 + .` 後按 Enter， 

```
rf> 2 17 + .
19  ok
rf> 
```
這兒發生了什麼事？首先 Forth 是直譯式語言，內建[直譯器](https://zh.wikipedia.org/wiki/%E7%9B%B4%E8%AD%AF%E5%99%A8) (英文：interpreter) 。當執行 `rf` 時，rtForth 會起動直譯器，先印出 `rf>`，等待使用者輸入，再一個字 (word) 一個字的，從輸入緩衝區 (input buffer) 掃描 (scan) 使用者的輸入，在字典 (word list) 中查詢字的定義 (definition) 並執行。完成後顯示 `ok`，告訴使用者執行成功。若失敗則印出錯誤訊息。然後再印出提示字串 (prompt) 請使用者繼續輸入。