#include "klyron_transpiler.h"

uint32_t klyron_transpiler_version(void) { return 1; }

klyron_transpiler_config_t klyron_transpiler_config_default(void) {
    klyron_transpiler_config_t config = { .enabled = true, .verbose = false };
    return config;
}
