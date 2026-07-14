#include "telemetry_config.h"
#include <stdlib.h>
#include <string.h>

void klyron_telemetry_config_set_version(klyron_telemetry_config_t* config, const char* version) {
    // version string managed externally
    config->version = version;
}
