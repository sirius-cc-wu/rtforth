# Windows下安裝 rtForth

目前動程科技並未提供 rtForth 的二進位執行檔。有興趣的人可以從 Github 下載原程式安裝。

* [https://github.com/mapacode/rtforth](https://github.com/mapacode/rtforth)

安裝步驟請見以下章節。

------------
## 安裝方法一：從雲端下載

請使用以下連結從 Google 雲端硬碟目錄下載。

* [rtForth 32 位元 Windows 版和 64 位元 Linux 版](https://drive.google.com/open?id=1c_q-Q2_iChoi9Y4lN9YTG8g6uyWtJULi)

目錄中有以下檔案：

* rf-win.exe：32 位元 Windows 版的 rtForth
* rf-linux：64 位元 Linux 版的 rtForth
* rtforth-vxx.xx-primer.pdf：本手冊

-------------------
## 安裝方法二：手動編譯

### 安裝 Rust 開發環境

`rtForth` 使用 `Rust` 語言進行開發。 在 `Windows` 上安裝 `Rust` 可以參考下列文章：

* [使用 Chocolatey 在 Win10 下配置 rust 开发环境](http://jamsa.github.io/shi-yong-chocolatey-zai-win10-xia-pei-zhi-rust-kai-fa-huan-jing.html)

另外，如果是基於 `Windows 10`，還可以使用 `WSL (Windows Subsystem for Linux)`，安裝方式跟Linux相同。

安裝 `Chocolatey`，要在管理者權限下啟動 `cmd.exe`，並執行下列：

```
@"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -InputFormat None -ExecutionPolicy Bypass -Command "iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))" && SET "PATH=%PATH%;%ALLUSERSPROFILE%\chocolatey\bin"
```

或是在管理者模式下啟動 `PowerShell.exe`，並執行下列：

```
Set-ExecutionPolicy Bypass -Scope Process -Force; iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))
```

需要透過 `rustup` 來安裝`rust`，要在管理者權限下啟動 `cmd.exe`，並執行下列：

```
# choco install rustup -pre -y
```

會輸出下列訊息：

```
Chocolatey v0.10.11
Installing the following packages:
rustup
By installing you accept licenses for the packages.
Progress: Downloading rustup 1.11.0-beta1... 100%

rustup v1.11.0-beta1
rustup package files install completed. Performing other installation steps.
Installing rustup-init...
info: syncing channel updates for 'stable-x86_64-pc-windows-msvc'
info: latest update on 2019-05-23, rust version 1.35.0 (3c235d560 2019-05-20)
info: downloading component 'rustc'
info: downloading component 'rust-std'
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: installing component 'rustc'
info: installing component 'rust-std'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: default toolchain set to 'stable'
rustup-init has been installed.
Only an exit code of non-zero will fail the package by default. Set
 `--failonstderr` if you want error messages to also fail a script. See
 `choco -h` for details.
Environment Vars (like PATH) have changed. Close/reopen your shell to
 see the changes (or in powershell/cmd.exe just type `refreshenv`).
 The install of rustup was successful.
  Software installed as 'exe', install location is likely default.

Chocolatey installed 1/1 packages.
 See the log for details (C:\ProgramData\chocolatey\logs\chocolatey.log).

```

此時，需要重新在管理者權限下啟動 `cmd.exe`，重新載入 `PATH`，這樣以下指令才會有作用。

目前編譯 `rtForth` 要使用 `nightly` 版的 `Rust`。因此，在管理者權限下啟動 `cmd.exe` 並執行下列：

```
# rustup toolchain add nightly
# rustup default nightly
```

--------------------------
### 下載 rtForth 原程式並編譯

在 `Windows` 下，還是透過 `Chocolatey` 來安裝 `git`，在管理者權限下啟動 `cmd.exe`，並執行下列：

```
# choco install git -y
```

此時，需要重新在管理者權限下啟動 `cmd.exe`，重新載入 `PATH`，這樣以下指令才會有作用。

使用 `git` 指令下載程式碼，

```
＃ git clone https://github.com/chengchangwu/rtforth.git
```

編譯除錯版的 `rtForth`：
```
＃ cargo build --example rf
```
編譯出來的除錯版 `rtForth` 位於 ./target/debug/examples/rf 。

編譯最佳化版的 `rtForth`：
```
＃ cargo build --example rf --release
```
編譯出來最佳化版的 `rtForth` 位於 ./target/release/examples/rf 。

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

| 指令  | 堆疊效果及指令說明         | 口語唸法 |
| ----- | -------------------------- | -------- |
| `bye` | ( -- ) &emsp; 離開 rtForth | bye      |