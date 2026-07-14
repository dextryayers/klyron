#include "klyron_pm.h"

uint32_t klyron_pm_version(void) { return 1; }

klyron_pm_config_t klyron_pm_config_default(void) {
    klyron_pm_config_t config = { .enabled = true, .verbose = false };
    return config;
}
