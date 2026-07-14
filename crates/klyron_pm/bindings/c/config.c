#include "config.h"
#include <stdlib.h>

klyron_pm_config_t klyron_pm_config_default(void) {
    klyron_pm_config_t config = { .enabled = true, .verbose = false };
    return config;
}
