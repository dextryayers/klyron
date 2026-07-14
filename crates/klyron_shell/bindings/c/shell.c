#include "shell.h"
#include <stdlib.h>
#include <string.h>

klyron_shell_config_t* klyron_shell_config_new(void) {
    klyron_shell_config_t* cfg = malloc(sizeof(klyron_shell_config_t));
    if (cfg) {
        cfg->version = "0.1.0";
    }
    return cfg;
}

void klyron_shell_config_free(klyron_shell_config_t* config) {
    free(config);
}

const char* klyron_shell_version(void) {
    return "klyron_shell@0.1.0";
}
