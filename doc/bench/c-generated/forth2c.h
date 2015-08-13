/*
 * forth2c.h
 *	Definitions for the Forth to C converter.
 *	(C) Martin Maierhofer 1994
 *	m.maierhofer@ieee.org
 */

#include <errno.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
                       
/*
 * _C_PAD_SIZE
 * The size of the pad buffer.
 */
#define _C_PAD_SIZE             200

/*
 * _C_PICBUFFER_SIZE
 * The size of the numeric pictured output buffer.
 */
#define _C_PICBUFFER_SIZE       100
 		 
/*
 * _C_????
 * These macros handle conversion from single cells to double cells
 * and vice versa. I'm not too sure whether they are correct.
 */
#define	__C_SHIFTAMOUNT	(sizeof(DCell) * 4)
#define _C_MAKEHIGH(d)	((DCell) (((DCell) (d)) << __C_SHIFTAMOUNT))
#define _C_UMAKEHIGH(d)	((UDCell) (((UDCell) (d)) << __C_SHIFTAMOUNT))
#define _C_HIGHHALF(d)	((Cell) (((DCell) (d)) >> __C_SHIFTAMOUNT))
#define _C_LOWHALF(d)	((Cell) (_C_HIGHHALF(_C_MAKEHIGH((Cell) (d)))))
#define _C_UHIGHHALF(d)	((UCell) (((UDCell) (d)) >> __C_SHIFTAMOUNT))
#define _C_ULOWHALF(d)	((UCell) (_C_UHIGHHALF(_C_UMAKEHIGH((UCell) (d)))))
#define _C_MAKEDCELL(d, h, l)	\
			((UDCell) (d) = _C_UMAKEHIGH(h) | (UCell) (l))
#define _C_MAKEUDCELL(ud, uh, ul)	\
			((UDCell) (ud) = _C_UMAKEHIGH(uh) | (UCell) (ul))			
 
/*
 * _C_MAX, _C_NOTZERO
 * Quite simple.
 */
#define _C_MAX(a, b)    ((a) < (b) ? (b) : (a))
#define _C_NOTZERO(a)   ((a) != 0 ? 1 : 0)

/*
 * _C_ALIGN
 * Takes a pointer to a Char and aligns it to the next Cell aligned
 * address.
 */
#define _C_ALIGN(a)     ((UCell) (a) += ((UCell) (a) % sizeof(Cell) ? \
                        sizeof(Cell) - (UCell) (a) % sizeof(Cell) : 0))


/**********************************************************************/
                                                     
/*
 * Some variables/functions for runtime support.
 */

extern int	_C_DATA_SIZE;	/* defined in the C output file */

extern Cell	_C_state;	/* the current state (== 0) */
extern Cell	_C_base;	/* the current base */
extern Char *	_C_data;	/* base pointer of data space */
extern Char *	_C_here;	/* represents 'here' */
extern int	_C_pic_buffer_index;	/* index into _C_pic_buffer */
				/* buffer for pictured numeric output */
extern char	_C_pic_buffer[_C_PICBUFFER_SIZE];
extern char	_C_pad[_C_PAD_SIZE];	/* the forth pad */
      
extern void _C_request_mem(Cell nrbytes);
extern Char * _C_fetch_here(void);
extern void _C_pic_add_char(char c);
extern void _C_pic_add_digit(int dig);
extern int _C_to_digit(Char d);

/**********************************************************************/

/*
 * These are the structs that are used to return values from functions
 * Note: there can be at most 20 Cells returned - this should be
 * sufficient for most applications. If you have to return more however,
 * the compiler dumps a warning message and you have to insert a
 * definition for the struct yourself !
 */
 
typedef struct
{
	Cell	cell0;
	Cell	cell1;
} Cells2;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
} Cells3;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
} Cells4;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
} Cells5;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
} Cells6;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
} Cells7;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
} Cells8;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
	Cell	cell8;
} Cells9;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
	Cell	cell8;
	Cell	cell9;
} Cells10;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
	Cell	cell8;
	Cell	cell9;
	Cell	cell10;
} Cells11;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
	Cell	cell8;
	Cell	cell9;
	Cell	cell10;
	Cell	cell11;
} Cells12;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
	Cell	cell8;
	Cell	cell9;
	Cell	cell10;
	Cell	cell11;
	Cell	cell12;
} Cells13;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
	Cell	cell8;
	Cell	cell9;
	Cell	cell10;
	Cell	cell11;
	Cell	cell12;
	Cell	cell13;
} Cells14;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
	Cell	cell8;
	Cell	cell9;
	Cell	cell10;
	Cell	cell11;
	Cell	cell12;
	Cell	cell13;
	Cell	cell14;
} Cells15;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
	Cell	cell8;
	Cell	cell9;
	Cell	cell10;
	Cell	cell11;
	Cell	cell12;
	Cell	cell13;
	Cell	cell14;
	Cell	cell15;
} Cells16;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
	Cell	cell8;
	Cell	cell9;
	Cell	cell10;
	Cell	cell11;
	Cell	cell12;
	Cell	cell13;
	Cell	cell14;
	Cell	cell15;
	Cell	cell16;
} Cells17;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
	Cell	cell8;
	Cell	cell9;
	Cell	cell10;
	Cell	cell11;
	Cell	cell12;
	Cell	cell13;
	Cell	cell14;
	Cell	cell15;
	Cell	cell16;
	Cell	cell17;
} Cells18;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
	Cell	cell8;
	Cell	cell9;
	Cell	cell10;
	Cell	cell11;
	Cell	cell12;
	Cell	cell13;
	Cell	cell14;
	Cell	cell15;
	Cell	cell16;
	Cell	cell17;
	Cell	cell18;
} Cells19;

typedef struct
{
	Cell	cell0;
	Cell	cell1;
	Cell	cell2;
	Cell	cell3;
	Cell	cell4;
	Cell	cell5;
	Cell	cell6;
	Cell	cell7;
	Cell	cell8;
	Cell	cell9;
	Cell	cell10;
	Cell	cell11;
	Cell	cell12;
	Cell	cell13;
	Cell	cell14;
	Cell	cell15;
	Cell	cell16;
	Cell	cell17;
	Cell	cell19;
} Cells20;


