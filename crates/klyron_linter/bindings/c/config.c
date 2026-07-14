#include "config.h"
#include <stdlib.h>

klyron_linter_config_t klyron_linter_config_default(void) {
    klyron_linter_config_t config = { .enabled = true, .verbose = false };
    return config;
}
