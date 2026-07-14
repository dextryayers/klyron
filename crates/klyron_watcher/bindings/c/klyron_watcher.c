#include "klyron_watcher.h"

uint32_t klyron_watcher_version(void) { return 1; }

klyron_watcher_config_t klyron_watcher_config_default(void) {
    klyron_watcher_config_t config = { .enabled = true, .verbose = false };
    return config;
}
