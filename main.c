#include "l1.h"
#include "l2.h"
#include <stdlib.h>
#include <stdio.h>
#include <signal.h>

volatile bool running = 1;

static void sighandler(int sig)
{
	(void)sig;
	running = 0;
}

static void setup_sighandler(void)
{
	struct sigaction sigact;
	sigact.sa_handler = sighandler;
	sigemptyset(&sigact.sa_mask);
	sigact.sa_flags = 0;
	sigaction(SIGINT,  &sigact, NULL);
	sigaction(SIGTERM, &sigact, NULL);
	sigaction(SIGQUIT, &sigact, NULL);
	sigaction(SIGPIPE, &sigact, NULL);
}

void *realtime_thread(void *arg)
{
	struct L2 *l2 = arg;
	struct L1 *l1 = l1_init();
	if (l1 == NULL) {
		running = 0;
		return NULL;
	}
	while (running) {
		int ret = l1_process(
			l1,
			(struct L1Callbacks) {
				.rx_burst = l2_rx_callback,
				.rx_burst_arg = l2,
				.tx_burst = l2_tx_callback,
				.tx_burst_arg = l2,
				// TODO: command callbacks
			}
		);
		if (ret < 0) {
			fprintf(stderr, "l1_process error: %d\n", ret);
			running = 0;
			break;
		}
	}
	l1_free(l1);
	return NULL;
}

int main(int argc, char *argv[])
{
	(void)argc; (void)argv;
	setup_sighandler();

	struct L2 *l2 = l2_init();

	// TODO: start realtime thread and do less timing critical
	// processing here in main thread.
	realtime_thread(l2);
	return 0;
}
