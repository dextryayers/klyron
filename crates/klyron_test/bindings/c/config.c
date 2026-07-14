#include "config.h"
#include <stdlib.h>

klyron_test_config_t klyron_test_config_default(void) {
    klyron_test_config_t config = { .enabled = true, .verbose = false };
    return config;
}
