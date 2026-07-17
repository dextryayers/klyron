#include "klyron_v8.h"
#include "cpp/impl/internal.h"

klyron_v8_result_t klyron_v8_get_heap_stats(klyron_v8_isolate_t* isolate,
                                             klyron_v8_heap_stats_t* stats) {
    klyron_v8_result_t result = {false, {0}};
    if (!isolate || !stats) return result;

    v8::HeapStatistics hs;
    isolate->isolate->GetHeapStatistics(&hs);

    stats->total_heap_size             = hs.total_heap_size();
    stats->total_heap_size_executable  = hs.total_heap_size_executable();
    stats->total_physical_size         = hs.total_physical_size();
    stats->total_available_size        = hs.total_available_size();
    stats->used_heap_size              = hs.used_heap_size();
    stats->heap_size_limit             = hs.heap_size_limit();
    stats->malloced_memory             = hs.malloced_memory();
    stats->peak_malloced_memory        = hs.peak_malloced_memory();
    stats->number_of_native_contexts   = hs.number_of_native_contexts();
    stats->number_of_detached_contexts = hs.number_of_detached_contexts();
    stats->total_global_handles_size   = hs.total_global_handles_size();
    stats->used_global_handles_size    = hs.used_global_handles_size();
    stats->external_memory             = hs.external_memory();

    set_bool_result(&result, true);
    return result;
}

void klyron_v8_low_memory_notification(klyron_v8_isolate_t* isolate) {
    if (isolate && isolate->isolate) {
        isolate->isolate->LowMemoryNotification();
    }
}

void klyron_v8_idle_notification(klyron_v8_isolate_t* isolate,
                                 double deadline_in_seconds) {
    if (isolate && isolate->isolate) {
        isolate->isolate->IdleNotificationDeadline(deadline_in_seconds);
    }
}

void klyron_v8_set_memory_pressure(klyron_v8_isolate_t* isolate,
                                   klyron_v8_memory_pressure_t pressure) {
    if (!isolate || !isolate->isolate) return;
    switch (pressure) {
        case KLYRON_V8_MEMORY_PRESSURE_MODERATE:
            isolate->isolate->MemoryPressureNotification(
                v8::MemoryPressureLevel::kModerate);
            break;
        case KLYRON_V8_MEMORY_PRESSURE_CRITICAL:
            isolate->isolate->MemoryPressureNotification(
                v8::MemoryPressureLevel::kCritical);
            break;
        default:
            break;
    }
}

void klyron_v8_request_gc(klyron_v8_isolate_t* isolate) {
    if (isolate && isolate->isolate) {
        isolate->isolate->RequestGarbageCollectionForTesting(
            v8::Isolate::kFullGCCallback);
    }
}

size_t klyron_v8_get_malloced_memory(klyron_v8_isolate_t* isolate) {
    if (!isolate || !isolate->isolate) return 0;
    return isolate->isolate->GetMallocedMemory();
}

size_t klyron_v8_adjust_external_memory(klyron_v8_isolate_t* isolate,
                                        int64_t change) {
    if (!isolate || !isolate->isolate) return 0;
    return isolate->isolate->AdjustAmountOfExternalAllocatedMemory(change);
}
