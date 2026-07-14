#include "template_builder.h"
#include <stdlib.h>

struct klyron_template_builder {
    const char* version;
};

klyron_template_builder_t* klyron_template_builder_new(void) {
    klyron_template_builder_t* b = malloc(sizeof(klyron_template_builder_t));
    if (b) b->version = "0.1.0";
    return b;
}

void klyron_template_builder_free(klyron_template_builder_t* builder) {
    free(builder);
}

void klyron_template_builder_set_version(klyron_template_builder_t* builder, const char* version) {
    if (builder) builder->version = version;
}

klyron_template_config_t* klyron_template_builder_build(klyron_template_builder_t* builder) {
    if (!builder) return NULL;
    klyron_template_config_t* cfg = klyron_template_config_new();
    if (cfg) cfg->version = builder->version;
    return cfg;
}
