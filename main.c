#include "l1.h"
#include "l2.h"
#include <stdlib.h>
#include <stdio.h>

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
	return NULL;
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
