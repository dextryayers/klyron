#include "klyron_v8.h"
#include "cpp/impl/internal.h"

klyron_v8_snapshot_t* klyron_v8_snapshot_create(klyron_v8_context_t* ctx) {
    (void)ctx;
    return nullptr;
}

klyron_v8_snapshot_t* klyron_v8_snapshot_load(const char* blob,
                                               size_t length) {
    if (!blob || length == 0) return nullptr;
    return new klyron_v8_snapshot(
        reinterpret_cast<const uint8_t*>(blob), length);
}

void klyron_v8_snapshot_dispose(klyron_v8_snapshot_t* snapshot) {
    delete snapshot;
}
