#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * JSC does not expose an "Isolate" concept in its C API.
 * The JSContextGroupRef serves a similar role — contexts within
 * the same group share an execution environment.
 * These stubs are kept for API parity with V8.
 */

void klyron_jsc_isolate_enter(klyron_jsc_engine_t* engine) {
    (void)engine;
}

void klyron_jsc_isolate_exit(klyron_jsc_engine_t* engine) {
    (void)engine;
}
