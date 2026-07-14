#include "config.h"
#include <stdlib.h>
#include <string.h>

klyron_config_config_t* klyron_config_config_new(void) {
    klyron_config_config_t* cfg = malloc(sizeof(klyron_config_config_t));
    if (cfg) {
        cfg->version = "0.1.0";
    }
    return cfg;
}

void klyron_config_config_free(klyron_config_config_t* config) {
    free(config);
}

const char* klyron_config_version(void) {
    return "klyron_config@0.1.0";
}
