/*
 * Sample GMUX glue carrying the C nits this decree knows.
 *
 * Three I/O protocols exist:
 *   Classic (PIO)  - pre-retina Macs: direct inb/outb at base+port.
 *   Indexed        - retina Macs (2012-2015): indexed port access.
 */

static void
sample_task(void *context)
{
	if ((uintptr_t)context != 0)
		sample_restore();
	else
		sample_off();
}

static int
sample_wait(uint16_t base)
{
	int i;

	for (i = 0; i < SAMPLE_RETRIES; i++) {
		if (inb(base) == 0)
			return (0);
		DELAY(SAMPLE_POLL_DELAY);
	}

	return (ETIMEDOUT);
}
