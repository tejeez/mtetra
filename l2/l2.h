#ifndef L2_H_INCLUDED
#define L2_H_INCLUDED
#include "l1.h"

struct L2;

// Allocate and initialize a new L2 instance
struct L2 *l2_init(void);

// Called by L1 once a slot.
// If burst(s) were received in the slot, they are passed in burst.
void l2_rx_callback(void *arg, int32_t carrier, struct SlotNumber slot, int64_t slot_time, const struct RxBurst *burst);

// Called by L1 once a slot.
// If a burst should be transmitted in the slot, write it to burst.
void l2_tx_callback(void *arg, int32_t carrier, struct SlotNumber slot, int64_t slot_time, struct TxBurst *burst);

#endif
