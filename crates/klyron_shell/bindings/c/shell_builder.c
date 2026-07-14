#include "shell_builder.h"
#include <stdlib.h>

struct klyron_shell_builder {
    const char* version;
};

klyron_shell_builder_t* klyron_shell_builder_new(void) {
    klyron_shell_builder_t* b = malloc(sizeof(klyron_shell_builder_t));
    if (b) b->version = "0.1.0";
    return b;
}

void klyron_shell_builder_free(klyron_shell_builder_t* builder) {
    free(builder);
}

void klyron_shell_builder_set_version(klyron_shell_builder_t* builder, const char* version) {
    if (builder) builder->version = version;
}

klyron_shell_config_t* klyron_shell_builder_build(klyron_shell_builder_t* builder) {
    if (!builder) return NULL;
    klyron_shell_config_t* cfg = klyron_shell_config_new();
    if (cfg) cfg->version = builder->version;
    return cfg;
}
