#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * JSC C API does not support snapshots.
 * These stubs return error/unimplemented.
 */

klyron_jsc_value_t* klyron_jsc_snapshot_create(klyron_jsc_engine_t* engine) {
    (void)engine;
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_snapshot_load(const char* blob, size_t length) {
    (void)blob;
    (void)length;
    return nullptr;
}

void klyron_jsc_snapshot_dispose(klyron_jsc_value_t* snapshot) {
    delete snapshot;
}
