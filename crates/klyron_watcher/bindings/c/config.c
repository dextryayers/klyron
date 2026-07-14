#include "config.h"
#include <stdlib.h>

klyron_watcher_config_t klyron_watcher_config_default(void) {
    klyron_watcher_config_t config = { .enabled = true, .verbose = false };
    return config;
}
