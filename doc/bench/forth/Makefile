FORTH=~/gforth/gforth

all:
	echo sieve
	time $(FORTH) ./sieve.fs -e "main bye"
	time $(FORTH) ./bubble-sort.fs -e "main bye"
	time $(FORTH) -m 200000 ./matrix-mult.fs -e "main bye"
	echo fib
	time $(FORTH) ./fib.fs -e "main bye"
	time $(FORTH) ~/ftp/bench/forth/ssieve-a.frt -e "?silent on list-primes 1 10000000 bye"
	time $(FORTH) -m 500000 ./mm-rtcg.fs -e "main bye"

	time $(FORTH) ./repeat.fs

clean:
