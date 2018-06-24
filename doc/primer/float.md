# 浮點運算

浮點數 (floating point) 是電腦用來表示實數的方式。浮點數不是實數，只能近似實數。這產生種種問題。因此，如果你是位工作上需要處理浮點數的工程師，務必閱讀以下文件：

* David Goldberg 的 [What Every Computer Scientist Should Know About Floating-Point Arithmetic](https://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html)

Forth 提供浮點運算能力，浮點運算類似整數運算，但有以下不同，

* 輸入浮點數必須有字母 `e`。字母 `e` 後的整數代表 10 的指數。若沒有整數，指數為 0。所以 `1e` 是 1.0 &times; 10<sup>0</sup> 也就是 1.0。2e3 是 2.0 &times; 10<sup>3</sup> 也就是 2000.0 。
以下都是正確的浮點輸入：`1e` `1.e` `1.e0` `+1.23e-1` `-1.23e+1`。
* 四則運算指令以 `f` 開頭。比如 `f+`、`f-`、`f*`、`f/`。
* 沒有和 `mod` 及 `/mod` 對應的指令，但有求次方的指令 `f**`，以及求平方根的 `fsqrt`。
* 資料堆疊放的是整數，浮點數放在另一個浮點堆疊 (floating point stack) 上。
* 浮點數的堆疊效果表示法是 ( F: before -- after )。

讓我們測試一下。

例一：輸入 `1e` `1.e` `1.e0` `+1.23e-1` `-1.23e+1` 並以 `f.` 印出來。
```
rf> 1e f.  1.e f.  1.e0 f.  +1.23e-1 f.  -1.23e+1 f.
1.0000000 1.0000000 1.0000000 0.1230000 -12.3000000  ok
```

例二：以浮點計算 7 + (6 &times; 5<sup>2</sup> + 3)
```
rf> 5e 2e f**  6e f*  3e f+  7e f+  f.
160.0000000  ok
```

例三：以浮點計算 1 / (3<sup>2</sup>)
```
rf> 1e  3e 2e f**  f/  f.
0.1111111  ok
```

例四：計算 2.0 的平方根。
```
rf> 2e fsqrt  f.
1.4142136  ok
```

### 浮點數不是實數

浮點數不是實數，只是近似實數。它能表示的實數範圍有限。精確度也有限。以下以例子呈現幾個浮點數的問題。以及當浮點數運算遇到 0/0 、 1/0 或是對負數開根號時的處理方式。這些例子在 64 位元的 rtforth 上測試。不同的電腦及不同的 Forth 會有不同的結果。

例五：計算 10<sup>10</sup>、 10<sup>100</sup> 及 10<sup>1000</sup>
```
rf> 1e10 f.
10000000000.0000000  ok
rf> 1e100 f.
10000000000000002101697803323328251387822715387464188032188166609887360023982790799717755191065313280.0000000  ok
rf> 1e1000 f.
inf  ok
```
從例子中可以看出，電腦無法正確表示 10<sup>100</sup> 次方，只能用一個近似的數字來表示。正確表示的的就只有前面 16 位數。而遇到 10<sup>1000</sup>時，電腦干脆以無限大 (inf) 來表示。

例六：計算 10<sup>100</sup> + 1。
```
rf> 1e100 1e f+ f.
10000000000000002101697803323328251387822715387464188032188166609887360023982790799717755191065313280.0000000  ok
```

從例子中可以看出計算 10<sup>100</sup> + 1 和計算 10<sup>100</sup> 結果相同。將一個很大的數字加上相對來說很小的數字時，很小的數字常被忽略不計。如果我們將 10<sup>100</sup> 加 1，連加 10<sup>100</sup> 次，照理應得到 2 &times; 10<sup>100</sup>。但結果還是 10<sup>100</sup>。也就是說，原本還有 16 位數有效位數，只剩 0 位數有效位數。

因此，為了計算的精確度，維持足夠的有效位數，當要將一群大小懸殊的數字相加時，應先加數值較小的數，再加數值大的數。 1 + 10 + 100 + 1000 + ... + 10<sup>100</sup> 會比 10<sup>100</sup> + 10<sup>99</sup> + ... + 10 + 1 精確得多。

例七：計算 -2.0 的平方根、 0.0 / 0.0 以及 1.0 / 0.0 。
```
rf> -2e fsqrt f.
NaN  ok
rf> 0e 0e f/ f.
NaN  ok
rf> 1e 0e f/ f.
inf  ok
```

1.0/0.0 得到無限大 (infinity)。但是 0.0/0.0 或是 1.0/0.0 會得到 NaN (Not a Number)。NaN 有個很重要的特性，對它進行的幾乎所有的算術運算都會得到 NaN。這簡化了浮點運算錯誤的處理，程式不必對每個有可能出錯的運算進行檢查，只需要檢查最後的結果，就知道是否有某浮點運算出錯。

例八：求 (1.0 / 0.0) &times; 0.0
```
rf> 1e 0e f/  0e f*  f.
NaN  ok
```
顯然，無限大乘以 0 會得到 NaN。同樣，無限大減無限大、無限大除以無限大都會得到 NaN。

### 本節指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `f.` | ( F:&nbsp; r -- ) &emsp; 印出浮點堆疊上最後的浮點數，並將它從堆疊上移除 | f-dot |
| `f+` | ( F:&nbsp; r1 r2 -- r1+r2 ) &emsp; 將浮點堆疊上的 r1 加上 r2，將結果放回浮點堆疊 | f-plus |
| `f-` | ( F:&nbsp; r1 r2 -- r1-r2 ) &emsp; 將浮點堆疊上的 r1 減去 r2，將結果放回浮點堆疊 | f-minus |
| `f*` | ( F:&nbsp; r1 r2 -- r1*r2 ) &emsp; 將浮點堆疊上的 r1 乘以 r2，將結果放回浮點堆疊 | f-star |
| `f/` | ( F:&nbsp; r1 r2 -- r1/r2 ) &emsp; 將浮點堆疊上的 r1 除以 r2，將結果放回浮點堆疊。 | f-slash |
| `f**` | ( F:&nbsp; r1 r2 -- r1<sup>r2</sup>) &emsp; 求浮點堆疊上的 r1 的 r2 次方，將結果放回浮點堆疊 | f-star-star |
| `fsqrt` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算 r1 的平方根 | f-sqrt |

-----------
## 更多的浮點算術指令

如同整數，Forth 提供了類似 `abs` 、 `negate` 、 `max` 、 `min` 的 `fabs` 、 `fnegate` 、 `fmax` 、 `fmin` 。

另外，因浮點數的特性，還提供了對浮點數行四捨五入的 `fround`、求小於等於浮點數的最大整數的 `floor` 及求大於等於浮點數的最小整數的 `fceil` 。這三個指令雖說是求整數，但結果仍以浮點數表示。

注意 `fceil` 指令不是 [FORTH 標準](https://forth-standard.org/standard/index) 中的指令，但存在於某些 Forth 系統中 (包括 rtForth)。

練習一下，

例九：100.0 / 9.0 後四捨五入。
```
rf> 100e 9e f/  fround  f.
11.0000000  ok
```

例十：請問 3.5 介於哪兩個整數之間？
```
rf> 3.5e floor f.  3.5e fceil f.
3.0000000 4.0000000  ok
```

例十一：(29 / 4.05) 和 7 哪一個比較大？
```
rf> 29e 4.05e f/  7e fmax  f.
7.1604938  ok
```

例十二：求中位法 -ａbs(3.2 - 1.7) 和 -1.4 的最小值。 
```
rf> 3.2e 1.7e f-  fabs  fnegate  -1.4e fmin  f.
-1.5000000  ok
```

### 本節指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `fnegate` | ( F:&nbsp; r1 -- r2 ) &emsp; 求 r1 的加法反元素。 | f-negate |
| `fabs` | ( F:&nbsp; r1 -- r2 ) &emsp; 求 r1 的絕對值 | f-abs |
| `fmax` | ( F:&nbsp; r1 r2 -- r3 ) &emsp; 求 r1 和 r2 中較大的數 | f-max |
| `fmin` | ( F:&nbsp; r1 r2 -- r3 ) &emsp; 求 r1 和 r2 中較小的數 | f-min |
| `fround` | ( F:&nbsp; r1 -- r2 ) &emsp; 將 r1 四捨五入，注意結果仍是浮點數 | f-round |
| `floor` | ( F:&nbsp; r1 -- r2 ) &emsp; 求小於等於 r1 的最大整數，注意結果仍是浮點數 | floor |
| `fceil` | ( F:&nbsp; r1 -- r2 ) &emsp; 求大於等於 r1 的最小整數，注意結果仍是浮點數 | f-ceil |

----------------
## 整數和浮點數轉換

Forth 提供整數和浮點數的轉換指令。當浮點數轉成整數時，小數部份會被*向零捨去* 。

例十三：將 2.9 及 -2.9 轉成整數。
```
rf> 2.9e f>s .  -2.9e f>s .
2 -2  ok
```

例十四：將整數 2 和 3 轉成浮點數後相除。
```
rf> 2 s>f  3 s>f  f/  f.
0.6666667  ok
```

| 指令 | 堆疊效果及指令 說明                        | 口語唸法 |
|-----|-----------------------------------------|--------|
| `s>f` | ( n -- ) ( F:&nbsp; -- r ) &emsp; 將整數 n 轉成浮點數 r | s-to-f |
| `f>s` | ( -- n ) ( F:&nbsp; r -- ) &emsp; 將浮點數 r 轉成整數 n | f-to-s |

---------------
## 三角函數 

Forth 提供了三角函數的指令集。這在處理幾何問題時非常有用。不過使用時要注意其定義域和值域的範圍。

例十五：驗證 sin(&pi; / 2) = 1。
```
rf> pi 2e f/  fsin  f.
1.0000000  ok
```

例十六：驗證 sin(1)<sup>2</sup> + cos(1)<sup>2</sup> = 1。
```
rf> 1e fsin 2e f**  1e fcos 2e f**  f+  f.
1.0000000  ok
```
例十七：求 tan(45&deg;)。
```
rf> pi 4e f/  ftan  f.
1.0000000  ok
```

例十八：求 sin<sup>-1</sup>(sin(&pi;)) 和 sin<sup>-1</sup>(sin(-&pi;/2)) 。
```
rf> pi fsin  fasin  f.
0.0000000  ok
rf> pi fnegate 2e f/  fsin  fasin  f.
-1.5707963  ok
```

`fasin` 的值域是 -&pi;/2 - &pi;/2，因此 sin<sup>-1</sup>(sin(&pi;)) 無法得到 &pi;。

例十九：求 cos<sup>-1</sup>(cos(&pi;)) 和 cos<sup>-1</sup>(cos(-&pi;)) 。
```
rf> pi fcos  facos  f.
3.1415927  ok
rf> pi fnegate  fcos  facos  f.
3.1415927  ok
```
`facos` 的值域是 0 - &pi;，因此 cos<sup>-1</sup>(cos(-&pi;)) 無法得到 -&pi;。

例二十：驗證 4*atan(1) = &pi;。
```
rf> 1e fatan  4e f*  f.
3.1415927  ok
```

`fasin` 、 `facos` 、 `fatan` 的值域都無法涵蓋整個 360&deg; 的範圍。
`fatan2` 則可以從一個二維向量得到角度。而和它對應的 `fsincos` 則可以從一個角度得到一個二維單位向量。

例二十一：求向量 (1, 0) 、 (0, -1) 的角度。注意 fatan2 的堆疊效應是 ( y x -- &theta; ) 。
```
rf> 0e 1e fatan2  f.
0.0000000  ok
rf> -1e 0e fatan2  f.
-1.5707963  ok
```

例二十二：求對應弳度 0 和 -1.5707963 的二維單位向量。注意 fsincos 的堆疊效應是 ( &theta; -- sin&theta; cos&theta; ) 。
```
rf> 0e fsincos f. f.
1.0000000 0.0000000  ok
rf> -1.5707963e fsincos f. f.
0.0000000 -1.0000000  ok
```

### 本節指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|--------|
| `pi` | ( F:&nbsp; -- pi ) &emsp; 將 PI 放上浮點堆疊 | pi |
| `fsin` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算弳度角 r1 的正弦 r2 | f-sine |
| `fcos` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算弳度角 r1 的餘弦 r2 | f-cos |
| `ftan` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算弳度角 r1 的正切 r2 | f-tan |
| `fsincos` | ( F:&nbsp; r1 -- r2 r3 ) &emsp; 計算弳度角 r1 的正弦 r2 及餘弦 r3 | f-sine-cos |
| `fasin` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算正弦為 r1 的弳度角 r2 | f-a-sine |
| `facos` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算餘弦為 r1 的弳度角 r2 | f-a-cos |
| `fatan` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算正切為 r1 的弳度角 r2 | f-a-tan |
| `fatan2` | ( F:&nbsp; r1 r2 -- r3 ) &emsp; 從向量 (r1, r2) 得其方向角的弳度 r3 | f-a-tan-two |

-------------
## 本章重點整理

* 浮點數 (floating point)
* 無限大 (infinity)
* NaN (Not a Number)

-------------------------------------
## 本章指令集

| 指令 | 堆疊效果及指令說明                        | 口語唸法 |
|-----|----------------------------------------|-------|
| `pi` | ( F:&nbsp; -- pi ) &emsp; 將 PI 放上浮點堆疊 | pi |
| `f.` | ( F:&nbsp; r -- ) &emsp; 印出浮點堆疊上最後的浮點數，並將它從堆疊上移除 | f-dot |
| `f+` | ( F:&nbsp; r1 r2 -- r1+r2 ) &emsp; 將浮點堆疊上的 r1 加上 r2，將結果放回浮點堆疊 | f-plus |
| `f-` | ( F:&nbsp; r1 r2 -- r1-r2 ) &emsp; 將浮點堆疊上的 r1 減去 r2，將結果放回浮點堆疊 | f-minus |
| `f*` | ( F:&nbsp; r1 r2 -- r1*r2 ) &emsp; 將浮點堆疊上的 r1 乘以 r2，將結果放回浮點堆疊 | f-star |
| `f/` | ( F:&nbsp; r1 r2 -- r1/r2 ) &emsp; 將浮點堆疊上的 r1 除以 r2，將結果放回浮點堆疊。 | f-slash |
| `f**` | ( F:&nbsp; r1 r2 -- r1<sup>r2</sup>) &emsp; 求浮點堆疊上的 r1 的 r2 次方，將結果放回浮點堆疊 | f-star-star |
| `fsqrt` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算 r1 的平方根 | f-sqrt |
| `fnegate` | ( F:&nbsp; r1 -- r2 ) &emsp; 求 r1 的加法反元素。 | f-negate |
| `fabs` | ( F:&nbsp; r1 -- r2 ) &emsp; 求 r1 的絕對值 | f-abs |
| `fmax` | ( F:&nbsp; r1 r2 -- r3 ) &emsp; 求 r1 和 r2 中較大的數 | f-max |
| `fmin` | ( F:&nbsp; r1 r2 -- r3 ) &emsp; 求 r1 和 r2 中較小的數 | f-min |
| `fround` | ( F:&nbsp; r1 -- r2 ) &emsp; 將 r1 四捨五入，注意結果仍是浮點數 | f-round |
| `floor` | ( F:&nbsp; r1 -- r2 ) &emsp; 求小於等於 r1 的最大整數，注意結果仍是浮點數 | floor |
| `fceil` | ( F:&nbsp; r1 -- r2 ) &emsp; 求大於等於 r1 的最小整數，注意結果仍是浮點數 | f-ceil |
| `s>f` | ( n -- ) ( F:&nbsp; -- r ) &emsp; 將整數 n 轉成浮點數 r | s-to-f |
| `f>s` | ( -- n ) ( F:&nbsp; r -- ) &emsp; 將浮點數 r 轉成整數 n | f-to-s |
| `fsin` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算弳度角 r1 的正弦 r2 | f-sine |
| `fcos` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算弳度角 r1 的餘弦 r2 | f-cos |
| `ftan` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算弳度角 r1 的正切 r2 | f-tan |
| `fsincos` | ( F:&nbsp; r1 -- r2 r3 ) &emsp; 計算弳度角 r1 的正弦 r2 及餘弦 r3 | f-sine-cos |
| `fasin` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算正弦為 r1 的弳度角 r2 | f-a-sine |
| `facos` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算餘弦為 r1 的弳度角 r2 | f-a-cos |
| `fatan` | ( F:&nbsp; r1 -- r2 ) &emsp; 計算正切為 r1 的弳度角 r2 | f-a-tan |
| `fatan2` | ( F:&nbsp; r1 r2 -- r3 ) &emsp; 從向量 (r1, r2) 得其方向角的弳度 r3 | f-a-tan-two |