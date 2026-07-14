#include "config.h"
#include <stdlib.h>

klyron_transpiler_config_t klyron_transpiler_config_default(void) {
    klyron_transpiler_config_t config = { .enabled = true, .verbose = false };
    return config;
}
