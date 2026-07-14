#include "workspace_builder.h"
#include <stdlib.h>

struct klyron_workspace_builder {
    const char* version;
};

klyron_workspace_builder_t* klyron_workspace_builder_new(void) {
    klyron_workspace_builder_t* b = malloc(sizeof(klyron_workspace_builder_t));
    if (b) b->version = "0.1.0";
    return b;
}

void klyron_workspace_builder_free(klyron_workspace_builder_t* builder) {
    free(builder);
}

void klyron_workspace_builder_set_version(klyron_workspace_builder_t* builder, const char* version) {
    if (builder) builder->version = version;
}

klyron_workspace_config_t* klyron_workspace_builder_build(klyron_workspace_builder_t* builder) {
    if (!builder) return NULL;
    klyron_workspace_config_t* cfg = klyron_workspace_config_new();
    if (cfg) cfg->version = builder->version;
    return cfg;
}
