#include "workspace.h"
#include <stdlib.h>
#include <string.h>

klyron_workspace_config_t* klyron_workspace_config_new(void) {
    klyron_workspace_config_t* cfg = malloc(sizeof(klyron_workspace_config_t));
    if (cfg) {
        cfg->version = "0.1.0";
    }
    return cfg;
}

void klyron_workspace_config_free(klyron_workspace_config_t* config) {
    free(config);
}

const char* klyron_workspace_version(void) {
    return "klyron_workspace@0.1.0";
}
