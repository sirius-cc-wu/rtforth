IP=0 是個很複雜的概念。應簡化。

instruction_pointer = 0

* 初始化時 (免不了要做初始化，0 是最好的值)
	* 初始化時
	* Exit 有可能會從返回堆疊拿到之前那個初始化的 0。
* Bug (這是 bug 要保護)
	* 故意寫出 branch 到 instruction_pointer = 0 的程式。
* Quit
	* 目前在 ip=0 的位置放了 quit，使得執行到 ip=0 時就會執行 quit。也就不會回到任何 Forth 程式。
* 代表 primitive word (可以考慮不用 ip=0 代表 primitive)
	* inner 在做完後設定為 0。
	* evaluate 在一開始時會設。
	* 在 evaluate 中，instruction_pointer = 0 代表是 primitive 指令。因此先設為 0，如果高階指令，會改為非 0。可以考慮用 Option<Exception> 來表示而不用 0。
	* 錯誤的返回堆疊操作。
* 多工用 (似乎沒有必要)
	* Pause 時會設 instruction_pointer 為 0。
* 測試用 (似乎可以不用)
	* inner_interpret 指定。

建議移除後三項意義。只保留 quit 的意義。也就是初始化就是進入 quit 。
