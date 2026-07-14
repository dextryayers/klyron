#include "compat.h"
#include <stdlib.h>
#include <string.h>

klyron_compat_config_t* klyron_compat_config_new(void) {
    klyron_compat_config_t* cfg = malloc(sizeof(klyron_compat_config_t));
    if (cfg) {
        cfg->version = "0.1.0";
    }
    return cfg;
}

void klyron_compat_config_free(klyron_compat_config_t* config) {
    free(config);
}

const char* klyron_compat_version(void) {
    return "klyron_compat@0.1.0";
}
