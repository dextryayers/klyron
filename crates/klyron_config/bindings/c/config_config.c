#include "config_config.h"
#include <stdlib.h>
#include <string.h>

void klyron_config_config_set_version(klyron_config_config_t* config, const char* version) {
    // version string managed externally
    config->version = version;
}
