#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * JSC uses JSGlobalContextRef as both the context and the global scope.
 * A single global context is created in init; these stubs maintain API parity.
 */

void klyron_jsc_context_enter(klyron_jsc_engine_t* engine) {
    (void)engine;
}

void klyron_jsc_context_exit(klyron_jsc_engine_t* engine) {
    (void)engine;
}
