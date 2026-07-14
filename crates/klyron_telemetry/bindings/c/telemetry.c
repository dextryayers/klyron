#include "telemetry.h"
#include <stdlib.h>
#include <string.h>

klyron_telemetry_config_t* klyron_telemetry_config_new(void) {
    klyron_telemetry_config_t* cfg = malloc(sizeof(klyron_telemetry_config_t));
    if (cfg) {
        cfg->version = "0.1.0";
    }
    return cfg;
}

void klyron_telemetry_config_free(klyron_telemetry_config_t* config) {
    free(config);
}

const char* klyron_telemetry_version(void) {
    return "klyron_telemetry@0.1.0";
}
