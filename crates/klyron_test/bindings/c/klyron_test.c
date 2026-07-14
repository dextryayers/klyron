#include "klyron_test.h"

uint32_t klyron_test_version(void) { return 1; }

klyron_test_config_t klyron_test_config_default(void) {
    klyron_test_config_t config = { .enabled = true, .verbose = false };
    return config;
}
