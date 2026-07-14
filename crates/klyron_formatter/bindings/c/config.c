#include "config.h"
#include <stdlib.h>

klyron_formatter_config_t klyron_formatter_config_default(void) {
    klyron_formatter_config_t config = { .enabled = true, .verbose = false };
    return config;
}
