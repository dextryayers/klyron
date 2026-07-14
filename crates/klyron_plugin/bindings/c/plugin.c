#include "plugin.h"
#include <stdlib.h>
#include <string.h>

klyron_plugin_config_t* klyron_plugin_config_new(void) {
    klyron_plugin_config_t* cfg = malloc(sizeof(klyron_plugin_config_t));
    if (cfg) {
        cfg->version = "0.1.0";
    }
    return cfg;
}

void klyron_plugin_config_free(klyron_plugin_config_t* config) {
    free(config);
}

const char* klyron_plugin_version(void) {
    return "klyron_plugin@0.1.0";
}
