#include "deploy.h"
#include <stdlib.h>
#include <string.h>

klyron_deploy_config_t* klyron_deploy_config_new(void) {
    klyron_deploy_config_t* cfg = malloc(sizeof(klyron_deploy_config_t));
    if (cfg) {
        cfg->version = "0.1.0";
    }
    return cfg;
}

void klyron_deploy_config_free(klyron_deploy_config_t* config) {
    free(config);
}

const char* klyron_deploy_version(void) {
    return "klyron_deploy@0.1.0";
}
