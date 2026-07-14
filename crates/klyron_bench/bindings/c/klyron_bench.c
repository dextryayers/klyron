#include "klyron_bench.h"

uint32_t klyron_bench_version(void) { return 1; }

klyron_bench_config_t klyron_bench_config_default(void) {
    klyron_bench_config_t config = { .enabled = true, .verbose = false };
    return config;
}
