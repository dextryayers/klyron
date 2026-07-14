#include "klyron_formatter.h"

uint32_t klyron_formatter_version(void) { return 1; }

klyron_formatter_config_t klyron_formatter_config_default(void) {
    klyron_formatter_config_t config = { .enabled = true, .verbose = false };
    return config;
}
