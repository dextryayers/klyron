#include "docker_config.h"
#include <stdlib.h>
#include <string.h>

void klyron_docker_config_set_version(klyron_docker_config_t* config, const char* version) {
    // version string managed externally
    config->version = version;
}
