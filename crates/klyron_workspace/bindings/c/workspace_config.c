#include "workspace_config.h"
#include <stdlib.h>
#include <string.h>

void klyron_workspace_config_set_version(klyron_workspace_config_t* config, const char* version) {
    // version string managed externally
    config->version = version;
}
