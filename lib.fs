variable base
10 base !
: ?dup 0 <> if dup then ;
: cr 10 emit ;
: space 32 emit ;
: spaces 0 begin 2dup > while 1+ space repeat 2drop ;
marker empty
