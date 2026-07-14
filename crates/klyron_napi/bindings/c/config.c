#include "config.h"
#include <stdlib.h>
#include <string.h>

klyron_napi_config_t* klyron_napi_config_default(void) {
    klyron_napi_config_t* config = calloc(1, sizeof(klyron_napi_config_t));
    if (config) {
        config->cache_enabled = true;
        config->path_count = 1;
        config->paths = calloc(1, sizeof(char*));
        config->paths[0] = strdup("node_modules");
    }
    return config;
}

void klyron_napi_config_free(klyron_napi_config_t* config) {
    if (config) {
        for (size_t i = 0; i < config->path_count; i++) free(config->paths[i]);
        free(config->paths);
        free(config);
    }
}
