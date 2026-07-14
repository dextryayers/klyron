#include "template.h"
#include <stdlib.h>
#include <string.h>

klyron_template_config_t* klyron_template_config_new(void) {
    klyron_template_config_t* cfg = malloc(sizeof(klyron_template_config_t));
    if (cfg) {
        cfg->version = "0.1.0";
    }
    return cfg;
}

void klyron_template_config_free(klyron_template_config_t* config) {
    free(config);
}

const char* klyron_template_version(void) {
    return "klyron_template@0.1.0";
}
