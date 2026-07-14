#include "adapter.h"
#include <stdlib.h>
#include <string.h>

klyron_adapter_config_t* klyron_adapter_config_new(void) {
    klyron_adapter_config_t* cfg = malloc(sizeof(klyron_adapter_config_t));
    if (cfg) {
        cfg->version = "0.1.0";
    }
    return cfg;
}

void klyron_adapter_config_free(klyron_adapter_config_t* config) {
    free(config);
}

const char* klyron_adapter_version(void) {
    return "klyron_adapter@0.1.0";
}
