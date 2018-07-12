# 多工、異常處理與文本解譯器

TODO

-------------------------------------
## 本章指令集

| 指令 | 堆疊效果及指令說明                          | 口語唸法 |
|-----|------------------------------------------|--------|
| `quit` | ( -- ) &emsp; | quit |
| `evaluate` | ( -- ) &emsp; | evaluate |
| `token-empty?` | ( -- ) &emsp; | token-empty |
| `compiling?` | ( -- ) &emsp; | compiling |
| `compile-token` | ( -- ) &emsp; | compile-token |
| `interpret-token` | ( -- ) &emsp; | interpret-token |
| `error?` | ( -- ) &emsp; | |
| `?stacks` | ( -- ) &emsp; | check-stacks |
| `parse-word` | ( -- ) &emsp; | parse-word |
| `abort` | ( -- ) &emsp; | abort |
| `leave-task` | ( -- ) &emsp; | leave-task |
| `enter-task` | ( -- ) &emsp; | enter-task |
| `handler!` | ( -- ) &emsp; | handler-store |
