#include "klyron_registry.h"

uint32_t klyron_registry_version(void) { return 1; }

klyron_registry_config_t klyron_registry_config_default(void) {
    klyron_registry_config_t config = { .enabled = true, .verbose = false };
    return config;
}
