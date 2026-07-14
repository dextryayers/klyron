#include "klyron_linter.h"

uint32_t klyron_linter_version(void) { return 1; }

klyron_linter_config_t klyron_linter_config_default(void) {
    klyron_linter_config_t config = { .enabled = true, .verbose = false };
    return config;
}
