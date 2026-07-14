#include "klyron_bundler.h"

uint32_t klyron_bundler_version(void) { return 1; }

klyron_bundler_config_t klyron_bundler_config_default(void) {
    klyron_bundler_config_t config = { .enabled = true, .verbose = false };
    return config;
}
