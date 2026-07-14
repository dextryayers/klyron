#include "docker.h"
#include <stdlib.h>
#include <string.h>

klyron_docker_config_t* klyron_docker_config_new(void) {
    klyron_docker_config_t* cfg = malloc(sizeof(klyron_docker_config_t));
    if (cfg) {
        cfg->version = "0.1.0";
    }
    return cfg;
}

void klyron_docker_config_free(klyron_docker_config_t* config) {
    free(config);
}

const char* klyron_docker_version(void) {
    return "klyron_docker@0.1.0";
}
