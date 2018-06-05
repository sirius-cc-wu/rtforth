# 安裝 rtForth

```
$ cargo run --example rf
```

```
rtForth v0.1.39, Copyright (C) 2017 Mapacode Inc.
Type 'bye' or press Ctrl-D to exit.
rf> 
```
在 `rf>` 的提示字串後輸入 `: hello ." Hello World!" ;` 後按 Enter。請注意不要忽略每一個空格。這會定義一個能印出「Hello World!」，名為 hello 的 FORTH 指令。 rtForth 回應 ok，並顯示新的提示字串 `rf>`。

```
rf> : hello ." Hello World!" ;
 ok
rf> 
```
在 `rf>` 的提示字串後輸入 `hello` 後按 Enter。rtForth 印出 Hello World! 後回應 ok，再次顯示新的提示字串`rf>`。
```
rf> hello
Hello World! ok
rf> 
```

最後輸入 bye，離開 rtForth。

```
rf> bye
```
