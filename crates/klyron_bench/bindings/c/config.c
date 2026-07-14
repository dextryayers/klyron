#include "config.h"
#include <stdlib.h>

klyron_bench_config_t klyron_bench_config_default(void) {
    klyron_bench_config_t config = { .enabled = true, .verbose = false };
    return config;
}
