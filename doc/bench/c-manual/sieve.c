/****************** Start NSIEVE C Source Code ************************/

/****************************************************************/
/*                          NSIEVE                              */
/*                     C Program Source                         */
/*          Sieve benchmark for variable sized arrays           */
/*                 Version 1.2, 03 Apr 1992                     */
/*             Al Aburto (aburto@marlin.nosc.mil)               */
/*                      ('ala' on BIX)                          */
/*                                                              */
/*                                                              */
/* This Sieve of Eratosthenes program works with variable size  */
/* arrays. It is a straight forward extension of the original   */
/* Gilbreath version ( Gilbreath, Jim. "A High-Level Language   */
/* Benchmark." BYTE, September 1981, p. 180, and also Gilbreath,*/ 
/* Jim and Gary. "Eratosthenes Revisited: Once More Through the */
/* Sieve." BYTE January 1983, p. 283 ). Unlike the Sieve of     */
/* Gilbreath, NSIEVE uses register long variables, pointers,and */ 
/* large byte arrays via 'malloc()'.  Maximum array size is     */
/* currently set at 2.56 MBytes but this can be increased or    */
/* decreased by changing the program LIMIT constant.  NSIEVE    */
/* provides error checking to ensure correct operation.  Timer  */
/* routines are provided for several different systems. NSIEVE  */
/* results won't generally agree with the Gilbreath Sieve       */
/* results because NSIEVE specifically uses register long       */
/* variables. NSIEVE, and Sieve, are programs designed          */
/* specifically to generate and printout prime numbers (positive*/ 
/* integers which have no other integral factor other than      */
/* itself and unity, as 2,3,5,7,11, ... ). NSIEVE does not      */
/* conduct the 'typical' instructions one might expect from the */
/* mythical 'typical program'. NSIEVE results can be used to    */
/* gain a perspective into the relative performance of different*/
/* computer systems, but they can not be used in isolation to   */
/* categorize the general performance capabilities of any       */
/* computer system (no single benchmark program currently can do*/
/* this).                                                       */
/*                                                              */
/* The program uses a maximum array size of 2.56 MBytes. You can*/
/* increase or lower this value by changing the 'LIMIT' define  */
/* from 9 to a higher or lower value.  Some systems (IBM PC's   */
/* and clones) will be unable to work beyond 'LIMIT = 3' which  */
/* corresponds to an array size of only 40,000 bytes. Be careful*/
/* when specifying LIMIT > 3 for these systems as the system may*/ 
/* crash or hang-up. Normally NSIEVE will stop program execution*/  
/* when 'malloc()' fails.                                       */
/*                                                              */
/* The maximum array size is given by:                          */
/*              size = 5000 * ( 2 ** LIMIT ).                   */
/*                                                              */
/* The array 'Number_Of_Primes[LIMIT]' is intended to hold the  */
/* correct number of primes found for each array size up to     */
/* LIMIT = 20, but the array is only currently defined up to    */
/* LIMIT = 12.                                                  */
/*                                                              */
/* Program outputs to check for correct operation:              */
/*    Array Size  LIMIT    Primes Found      Last Prime         */
/*     (Bytes)                                                  */
/*         8191       0            1899           16381         */
/*        10000       1            2261           19997         */
/*        20000       2            4202           39989         */
/*        40000       3            7836           79999         */
/*        80000       4           14683          160001         */
/*       160000       5           27607          319993         */
/*       320000       6           52073          639997         */
/*       640000       7           98609         1279997         */
/*      1280000       8          187133         2559989         */
/*      2560000       9          356243         5119997         */
/*      5120000      10          679460        10239989         */
/*     10240000      11         1299068        20479999         */
/*     20480000      12         2488465        40960001         */
/*     40960000      13         -------        --------         */
/****************************************************************/

/****************************************************/
/* Example Compilation:                             */
/* (1) UNIX Systems:                                */
/*     cc -O -DUNIX nsieve.c -o nsieve              */
/*     cc -DUNIX nsieve.c -o nsieve                 */
/****************************************************/

#include <stdio.h>
#ifndef vax
#include <stdlib.h>
#endif
#include <math.h>
                     /***********************************************/
#define LIMIT 9      /* You may need to change this to '3' for PC's */
                     /* and Clones or you can experiment with higher*/
                     /* values, but '12' is currently the max.      */
		     /***********************************************/

                     /***********************************************/
                     /* You may just want to uncomment one of these */
                     /* to access the correct timer for your system.*/
                     /***********************************************/
/* #define UNIX      */
/* #define UNIX_Old  */
/* #define Amiga     */
/* #define BORLAND_C */
/* #define MSC       */

#ifdef Amiga
#include <exec/types.h>
#include <ctype.h>
#endif

#ifdef BORLAND_C
#include <ctype.h>
#include <dos.h>
#endif

#ifdef MSC
#include <ctype.h>
#endif

#ifndef TRUE
#define TRUE 1
#define FALSE 0
#endif

float nulltime,runtime,TimeArray[4];
float reftime,adjtime1,emips;
float hmips,lmips,smips[21];

long L_Prime,N_Prime;      /* Last Prime and Number of Primes Found */
long ErrorFlag;

long Number_Of_Primes[21]; /* List of Correct Number of Primes for */
                           /* each sieve array size.               */

long NLoops[21];

#ifdef xxxxxxxxxx
void main()
{

long  i,j,k,p;
float sumtime;


printf("\n   Sieve of Eratosthenes (Scaled to 10 Iterations)\n");
printf("   Version 1.2, 03 April 1992\n\n");
printf("   Array Size   Number   Last Prime    Linear");       
printf("     RunTime    MIPS\n");
printf("    (Bytes)   of Primes               Time(sec)");      
printf("    (Sec)\n");

        
                    /*******************************/
                    /* Number of        Array Size */
                    /* Primes Found      (Bytes)   */
Number_Of_Primes[0] =     1899;      /*       8191 */
Number_Of_Primes[1] =     2261;      /*      10000 */
Number_Of_Primes[2] =     4202;      /*      20000 */
Number_Of_Primes[3] =     7836;      /*      40000 */
Number_Of_Primes[4] =    14683;      /*      80000 */
Number_Of_Primes[5] =    27607;      /*     160000 */
Number_Of_Primes[6] =    52073;      /*     320000 */
Number_Of_Primes[7] =    98609;      /*     640000 */
Number_Of_Primes[8] =   187133;      /*    1280000 */
Number_Of_Primes[9] =   356243;      /*    2560000 */
Number_Of_Primes[10]=   679460;      /*    5120000 */
Number_Of_Primes[11]=  1299068;      /*   10240000 */
Number_Of_Primes[12]=  2488465;      /*   20480000 */
Number_Of_Primes[13]=        0;      /*   40960000 */
Number_Of_Primes[14]=        0;      /*   81920000 */
Number_Of_Primes[15]=        0;      /*  163840000 */

j = 8191;
k = 256;
p = 0;
SIEVE(j,k,p);

for( i=1 ; i<= 20 ; i++)
{
 NLoops[i] = 1;
}

p = 8;
if ( runtime > 0.125 ) p = 1;

NLoops[0] = 256 * p; 
NLoops[1] = 256 * p; 
NLoops[2] = 128 * p;
NLoops[3] =  64 * p;
NLoops[4] =  32 * p;
NLoops[5] =  16 * p;
NLoops[6] =   8 * p;
NLoops[7] =   4 * p;
NLoops[8] =   2 * p;
NLoops[9] =       p;
NLoops[10] =  p / 2;
NLoops[11] =  p / 4;

if ( p == 1 )
{
NLoops[10] = 1;
NLoops[11] = 1;
}

sumtime = 0.0;
i = 0;
j = 8191;
k = NLoops[0];
SIEVE(j,k,p);
sumtime = sumtime + runtime;
smips[i] = emips;

j = 5000;
ErrorFlag = 0;

for( i=1 ; i<= LIMIT ; i++)
{
   j = 2 * j;

   k = NLoops[i];

   SIEVE(j,k,p);
   smips[i] = emips;

   if( ErrorFlag == 0L )
   {
   if( N_Prime != Number_Of_Primes[i] )
   {
   printf("\n   Error --- Incorrect Number of Primes for Array: %ld\n",j);
   printf("   Number of  Primes  Found is: %ld\n",N_Prime);
   printf("   Correct Number of Primes is: %ld\n",Number_Of_Primes[i]);
   ErrorFlag = 1L;
   }
   }

   if( ErrorFlag > 0L ) break;

   sumtime = sumtime + runtime * ( 8191.0 / (float)j );

}

if( ErrorFlag == 2L )
{
printf("\n   Could Not Allocate Memory for Array Size: %ld\n",j);
}

sumtime = sumtime / (float)i;

j = LIMIT;
if( ErrorFlag == 1L) j = LIMIT - 1;

hmips = 0.0;
lmips = 1.0e+06;
for( i=0 ; i <= j ; i++)
{
if( smips[i] > hmips ) hmips = smips[i];
if( smips[i] < lmips ) lmips = smips[i];
}

printf("\n   Relative to 10 Iterations and the 8191 Array Size:\n");
printf("   Average RunTime = %8.3f (sec)\n",sumtime);
printf("   High  MIPS      = %8.1f\n",hmips);
printf("   Low   MIPS      = %8.1f\n\n",lmips);

}
#endif

main()
{
  SIEVE(8191,1000,0);
}

/**************************************/
/*  Sieve of Erathosthenes Program    */
/**************************************/

SIEVE(m,n,p)
long m,n,p;
{

register char *flags;
register long i,prime,k,ci;
register long count,size;

long  iter,j;

char *ptr;

#ifdef vax
char *malloc();
int   free(); 
#endif

size  = m - 1;
ptr   = malloc(m);

   ErrorFlag = 0L;
   N_Prime   = 0L;
   L_Prime   = 0L;

   if( !ptr )
     {
     ErrorFlag = 2L;
     return 0;
     }

   flags = ptr;

   dtime(TimeArray);
   dtime(TimeArray);
   nulltime = TimeArray[1];
   if ( nulltime < 0.0 ) nulltime = 0.0;

   j = 0;
                                                   /****************/
                                                   /* Instructions */
						   /*    *iter     */ 
                                                   /****************/
   dtime(TimeArray);
   for(iter=1 ; iter<=n ; iter++)                  
   {
   count = 0;                                        /* 1       */

   for(i=0 ; i<=size ; i++)                          /* 2       */
   {
   *(flags+i) = TRUE;                                /* 1*size  */
   }                                                 /* 3*size  */
                                                     /* 1       */
   ci = 0;                                           /* 1       */
     for(i=0 ; i<=size ; i++)                        /* 1       */
     {
       if(*(flags+i))                                /* 2*size  */
       {                                             /* 1*count */
       count++;                                      /* 1*count */
       prime = i + i + 3;                            /* 3*count */
         for(k = i + prime ; k<=size ; k+=prime)     /* 3*count */
         {
         ci = ci + 1;                                /* 1*ci    */
         *(flags+k)=FALSE;                           /* 1*ci    */
         }                                           /* 3*ci    */
                                                     /* 1*count */
       }
     }                                               /* 3*size  */
                                                     /* 1       */
   j = j + count;                                    /* 1       */
   }                                                 /* 4       */
   dtime(TimeArray);

   free(ptr);

   runtime = (TimeArray[1] - nulltime) * 10.0 / (float)n;

   if ( m == 8191 ) reftime = runtime;

   adjtime1 = reftime * ( (float)m / 8191.0 );

   emips = 9.0*(float)size+9.0*(float)count;
   emips = emips+5.0*(float)ci;
   emips = 1.0e-05*(emips/runtime);

   N_Prime = j / n;
   L_Prime = prime;

   if ( p != 0L )
   {
   printf("  %9ld   %8ld     %8ld  ",m,N_Prime,L_Prime);
   printf("%9.3f  %9.3f  %6.1f\n",adjtime1,runtime,emips);
   }

return 0;
}

/*********************************************************/
/* 'dtime()' outputs the elapsed time in seconds in p[1] */
/* from the first call to 'dtime()' to the next call.    */
/*********************************************************/

/******************************************************/
/* dtime function for the UNIX 'getrusage()' routine. */
/******************************************************/
#ifdef UNIX
#include <sys/time.h>
#include <sys/resource.h>

#ifdef hpux
#include <sys/syscall.h>
#define getrusage(a,b) syscall(SYS_getrusage,a,b)
#endif

struct rusage rusage;

dtime(p)
float p[];
{
   float q;

   q = p[2];

   getrusage(RUSAGE_SELF,&rusage);

   p[2] = (float)(rusage.ru_utime.tv_sec);
   p[2] = p[2] + (float)(rusage.ru_utime.tv_usec) / 1.0e+06;
   p[1] = p[2] - q;
 
   return 0;
}
#endif

/*****************************************************/
/* dtime function for the old UNIX 'times' routine.  */
/*****************************************************/
#ifdef UNIX_Old
#include <sys/times.h>
#include <sys/param.h>

#ifndef HZ
#define HZ 60
#endif

struct tms tms;

dtime(p)
float p[];
{

   float q;

   q = p[2];
   times(&tms);
   p[2] = (float)(tms.tms_utime) / (float)HZ;
   p[1] = p[2] - q;
   return 0;
}
#endif


/**********************************/
/*  dtime function for the Amiga  */
/**********************************/
#ifdef Amiga

#define HZ 50

dtime(p)
float p[];
{
   float q;

   struct tt {
      long days;
      long minutes;
      long ticks;
   } tt;

   q = p[2];

   DateStamp(&tt);

   p[2] = ((float)(tt.ticks+(tt.minutes*60L*(long)HZ)))/(float)HZ;
   p[1] = p[2] - q;
   return 0;
}
#endif


/***************************************************/
/*  dtime for IBM PC/PC-AT systems with Borland C  */
/***************************************************/
#ifdef BORLAND_C
#include <time.h>

#define HZ 100
struct time now;

dtime(p)
float p[];
{
   float q,v;

   q = p[2];

   gettime(&now);

   v = 60.0*(float)(now.ti_min);
   v = v +  (float)(now.ti_sec);
   v = v +  (float)(now.ti_hund) / (float)HZ;

   p[2] = v;
   p[1] = v - q;
   return 0;
}
#endif

/*****************************************************/
/* dtime() for IBM PC/PC-AT Systems with Microsoft C */
/*****************************************************/
#ifdef MSC
#include <time.h>

#define HZ CLK_TCK
clock_t tnow;

dtime(p)
float p[];
{
   float q;

   q = p[2];
   tnow = clock();
   p[2] = (float)tnow / (float)HZ;
   p[1] = p[2] - q;
   return 0;
}
#endif

/********************* End Of NSIEVE C Source Code *********************/
