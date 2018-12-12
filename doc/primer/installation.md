# 安裝 rtForth

目前動程科技並未提供 rtForth 的二進位執行檔。有興趣的人可以從 Github 下載原程式安裝。

* [https://github.com/mapacode/rtforth](https://github.com/mapacode/rtforth)

安裝步驟請見以下章節。

------------
## 安裝方法一：從雲端下載

請使用以下連結從 Google 雲端硬碟目錄下載。

[rtForth 32 位元 Windows 版和 64 位元 Linux 版](https://drive.google.com/open?id=1c_q-Q2_iChoi9Y4lN9YTG8g6uyWtJULi)

目錄中有以下檔案：

* rf-win.exe：32 位元 Windows 版的 rtForth
* rf-linux：64 位元 Linux 版的 rtForth
* rtforth-vxx.xx-primer.pdf：本手冊

-------------------
## 安裝方法二：手動編譯

### 安裝 Rust 開發環境

rtForth 使用 Rust 語言進行開發。 安裝 Rust 請參考

* [The Rust Programming Language 第一章](https://doc.rust-lang.org/book/second-edition/ch01-01-installation.html)

目前編譯 rtForth 要使用 nightly 版的 Rust。因此，

```
$ rustup default nightly
```

-------------------------
### 下載 rtForth 原程式並編譯

在 linux 下

```
$ git clone https://github.com/chengchangwu/rtforth.git
```

在 Windows 下請使用您熟悉的 git 工具。

編譯除錯版的 rtForth：
```
$ cargo build --example rf
```
編譯出來的除錯版 rtForth 位於 ./target/debug/examples/rf 。

編譯最佳化版的 rtForth：
```
$ cargo build --example rf --release
```
編譯出來最佳化版的 rtForth 位於 ./target/release/examples/rf 。

---------------
## Hello World!

執行 ./target/debug/examples/rf 或 ./target/release/examples/rf，出現以下訊息，

```
$ ./target/debug/examples/rf
rtForth v0.5.0, Copyright (C) 2018 Mapacode Inc.
Type 'bye' or press Ctrl-D to exit.
rf> 
```
在提示字串 `rf>` 之後輸入 `: hello ." Hello World!" ;` 後按 Enter。請注意不要忽略每一個空格。這會定義一個能印出「Hello World!」，名為 hello 的 FORTH 指令。 rtForth 回應 `ok`，並顯示新的提示字串 `rf>`。

```
rf> : hello ." Hello World!" ;
 ok
rf> 
```
在 `rf>` 之後輸入 `hello` 後按 Enter。rtForth 印出 Hello World! 後回應 `ok`，再次顯示新的提示字串`rf>`。
```
rf> hello
Hello World! ok
rf> 
```

最後輸入 `bye`，按 Enter 離開 rtForth。

```
rf> bye
```

------------
## 本章指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `bye` | ( -- ) &emsp; 離開 rtForth | bye   |
