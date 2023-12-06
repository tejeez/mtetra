#include <stdlib.h>
#include <stdio.h>
#include "l1.h"

struct L2 {
	// TODO
};

struct L2 *l2_init(void)
{
	struct L2 *l2 = calloc(1, sizeof(struct L2));
	// TODO
	return l2;
}

// Called by L1 once a slot.
// If burst(s) were received in the slot, they are passed in burst.
void l2_rx_callback(void *arg, struct SlotNumber slot, const struct RxBurst *burst)
{
	struct L2 *l2 = arg;
	fprintf(stderr, "RX callback for slot %2d,%2d,%2d\n", slot.multiframe, slot.frame, slot.timeslot);
	// TODO
}

// Called by L1 once a slot.
// If a burst should be transmitted in the slot, write it to burst.
void l2_tx_callback(void *arg, struct SlotNumber slot, struct TxBurst *burst)
{
	struct L2 *l2 = arg;
	fprintf(stderr, "TX callback for slot %2d,%2d,%2d\n", slot.multiframe, slot.frame, slot.timeslot);

	// Make some TX burst for testing
	burst->tag = TX_BURST_DL;
	size_t i;
	for (i = 0; i < sizeof(burst->dl); i++) {
		burst->dl[i] = (i >> 6) & 1;
	}
}

void *realtime_thread(void *arg)
{
	struct L2 *l2 = arg;
	struct L1 *l1 = l1_init();
	for (;;) {
		int ret = l1_process(
			l1,
			(struct L1Callbacks) {
				.rx_cb = l2_rx_callback,
				.rx_cb_arg = l2,
				.tx_cb = l2_tx_callback,
				.tx_cb_arg = l2,
			}
		);
		if (ret < 0) {
			fprintf(stderr, "l1_process error: %d\n", ret);
			break;
		}
	}
}

int main(int argc, char *argv[])
{
	(void)argc; (void)argv;

	struct L2 *l2 = l2_init();

	// TODO: start realtime thread and do less timing critical
	// processing here in main thread.
	realtime_thread(l2);
	return 0;
}
