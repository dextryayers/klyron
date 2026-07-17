#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * JSC C API does not expose heap statistics (no JSHeapStats equivalent).
 * This is a stub that returns minimal information.
 */

klyron_jsc_result_t klyron_jsc_get_heap_stats(klyron_jsc_engine_t* engine,
                                                klyron_jsc_heap_stats_t* stats) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !stats) return result;

    std::memset(stats, 0, sizeof(*stats));
    jsc_set_bool_result(&result, true);
    return result;
}

void klyron_jsc_request_gc(klyron_jsc_engine_t* engine) {
    (void)engine;
    /* JSC C API does not expose GC control */
}

void klyron_jsc_low_memory_notification(klyron_jsc_engine_t* engine) {
    (void)engine;
    /* JSC C API does not expose low memory notification */
}
