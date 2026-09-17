/*
 * Pre-C99 spellings the freebsd decree should reject.
 */
#include <sys/param.h>
#include <sys/types.h>

#define	TRUE	1
#define	FALSE	0
#define	dprintf(fmt, args...)	printf(fmt, args)

static __inline u_int32_t
mask_of(u_int32_t v)
{
	register int i;

	for (i = 0; i < 32; i++)
		v |= v >> i;
	return (v);
}

int
legacy_sum(a, b)
	int a;
	int b;
{
	return (a + b);
}

static int
no_prototype();

quad_t
widen()
{
	auto int scratch = 0;

	return ((quad_t)scratch);
}
