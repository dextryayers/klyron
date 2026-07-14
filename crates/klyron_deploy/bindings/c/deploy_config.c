#include "deploy_config.h"
#include <stdlib.h>
#include <string.h>

void klyron_deploy_config_set_version(klyron_deploy_config_t* config, const char* version) {
    // version string managed externally
    config->version = version;
}
