#include <stdlib.h>
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
	// TODO
}

// Called by L1 once a slot.
// If a burst should be transmitted in the slot, write it to burst.
void l2_tx_callback(void *arg, struct SlotNumber slot, struct TxBurst *burst)
{
	struct L2 *l2 = arg;
	// TODO
}

void *realtime_thread(void *arg)
{
	struct L2 *l2 = arg;
	struct L1 *l1 = l1_init();
	for (;;) {
		l1_process(
			l1,
			l2_rx_callback, l2,
			l2_tx_callback, l2
		);
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
