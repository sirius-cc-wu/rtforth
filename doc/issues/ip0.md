IP=0 是個很複雜的概念。應簡化。

instruction_pointer = 0 應該就只代表 idle。

以下有幾種可能造成 ip=0：
* 初始化時 (免不了要做初始化，0 是最好的值)
	* 初始化時
	* Exit 有可能會從返回堆疊拿到之前那個初始化的 0。
* Halt
	* 目前在 ip=0 的位置放了 halt，使得執行到 ip=0 時就會執行 halt。Halt 會設 ip=0，同時回傳 Quit，以脫離 inner interpreter。
* is_idle
	* 當 ip=0，VM 是在 idle 狀態。ip=0就代表 idle。idle 時，return stack 不一定會是空的。這是因為 halt 會設 ip=0 並不清 return stack。應考慮修改使得 ip=0 時，return stack 就是空的。
* 錯誤的返回堆疊操作。
