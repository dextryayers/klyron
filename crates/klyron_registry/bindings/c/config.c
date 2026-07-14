#include "config.h"
#include <stdlib.h>

klyron_registry_config_t klyron_registry_config_default(void) {
    klyron_registry_config_t config = { .enabled = true, .verbose = false };
    return config;
}
