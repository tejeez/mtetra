#include "l2.h"

#include <stdio.h>

// Struct to contain all state of L2
struct L2 {
	// TODO
};

struct L2 *l2_init(void)
{
	struct L2 *l2 = calloc(1, sizeof(struct L2));
	// TODO
	return l2;
}

void l2_rx_callback(void *arg, int32_t carrier, struct SlotNumber slot, int64_t slot_time, const struct RxBurst *burst)
{
	struct L2 *l2 = arg;
	fprintf(stderr, "RX callback for carrier %2d slot %2d,%2d,%2d\n", carrier, slot.multiframe, slot.frame, slot.timeslot);
	// TODO
}

void l2_tx_callback(void *arg, int32_t carrier, struct SlotNumber slot, int64_t slot_time, struct TxBurst *burst)
{
	struct L2 *l2 = arg;
	fprintf(stderr, "TX callback for carrier %2d slot %2d,%2d,%2d\n", carrier, slot.multiframe, slot.frame, slot.timeslot);

	// Make some TX burst for testing.
	// Transmit random bits to see if spectrum looks correct.
	burst->tag = TX_BURST_DL;
	size_t i;
	for (i = 0; i < sizeof(burst->dl); i++) {
		burst->dl[i] = (rand() >> 30) & 1;
	}
}
