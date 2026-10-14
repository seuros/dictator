/*
 * The same glue with every nit fixed, plus the cases that must stay quiet:
 * a counter inspected after its loop, and an integer smuggled through a
 * void pointer and unpacked rather than compared.
 *
 * Three I/O protocols exist:
 *   Classic (PIO)  - pre-Retina Macs: direct inb/outb at base+port.
 *   Indexed        - Retina Macs (2012-2015): indexed port access.
 */

static void
sample_task(void *context)
{
	if (context != NULL)
		sample_restore();
	else
		sample_off();
}

static int
sample_wait(uint16_t base)
{

	for (int i = 0; i < SAMPLE_RETRIES; i++) {
		if (inb(base) == 0)
			return (0);
		DELAY(SAMPLE_POLL_DELAY);
	}

	return (ETIMEDOUT);
}

static int
sample_scan(int n)
{
	int i;

	for (i = 0; i < n; i++) {
		if (sample_ready(i))
			break;
	}

	if (i == n)
		return (ENXIO);

	return (0);
}

static int
sample_unpack(void *context)
{

	return ((int)(uintptr_t)context);
}
