# 安裝 rtForth

目前動程科技並未提供 rtForth 的二進位執行檔。有興趣的人可以從 Github 下載原程式安裝。

* [https://github.com/chengchangwu/rtforth](https://github.com/chengchangwu/rtforth)

安裝步驟如下。

-------------------
## 安裝 Rust 開發環境

rtForth 使用 Rust 語言進行開發。 安裝 Rust 請參考

* [The Rust Programming Language 第一章](https://doc.rust-lang.org/book/second-edition/ch01-01-installation.html)

目前編譯 rtForth 要使用 nightly 版的 Rust。因此，

```
$ rustup default nightly
```

-------------------------
## 下載 rtForth 原程式並編譯

在 linux 下

```
$ git clone https://github.com/chengchangwu/rtforth.git
```

在 Windows 下請使用您熟悉的 git 工具。

編譯除錯版的 rtForth：
```
$ cargo build
```
編譯出來的除錯版 rtForth 位於 ./target/debug/rf 。

編譯最佳化版的 rtForth：
```
$ cargo build --release
```
編譯出來最佳化版的 rtForth 位於 ./target/release/rf 。

---------------
## Hello World!

執行 ./target/debug/rf 或 ./target/release/rf，出現以下訊息，

```
$ ./target/debug/rf
rtForth v0.3.0, Copyright (C) 2018 Mapacode Inc.
Type 'bye' or press Ctrl-D to exit.
rf> 
```
在 `rf>` 的提示字串後輸入 `: hello ." Hello World!" ;` 後按 Enter。請注意不要忽略每一個空格。這會定義一個能印出「Hello World!」，名為 hello 的 FORTH 指令。 rtForth 回應 `ok`，並顯示新的提示字串 `rf>`。

```
rf> : hello ." Hello World!" ;
 ok
rf> 
```
在 `rf>` 的提示字串後輸入 `hello` 後按 Enter。rtForth 印出 Hello World! 後回應 `ok`，再次顯示新的提示字串`rf>`。
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
