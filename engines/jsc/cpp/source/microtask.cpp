#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * JSC C API does not expose microtask queue control.
 * Microtasks are drained automatically by JSC after script execution.
 */

void klyron_jsc_microtasks_perform_check(klyron_jsc_engine_t* engine) {
    (void)engine;
    /* JSC drains microtasks automatically */
}
