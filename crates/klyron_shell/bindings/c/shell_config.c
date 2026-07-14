#include "shell_config.h"
#include <stdlib.h>
#include <string.h>

void klyron_shell_config_set_version(klyron_shell_config_t* config, const char* version) {
    // version string managed externally
    config->version = version;
}
