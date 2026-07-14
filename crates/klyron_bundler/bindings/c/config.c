#include "config.h"
#include <stdlib.h>

klyron_bundler_config_t klyron_bundler_config_default(void) {
    klyron_bundler_config_t config = { .enabled = true, .verbose = false };
    return config;
}
