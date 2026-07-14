#ifndef KLYRON_PLUGIN_BUILDER_H
#define KLYRON_PLUGIN_BUILDER_H

#include "plugin.h"

typedef struct klyron_plugin_builder klyron_plugin_builder_t;

klyron_plugin_builder_t* klyron_plugin_builder_new(void);
void klyron_plugin_builder_free(klyron_plugin_builder_t* builder);
void klyron_plugin_builder_set_version(klyron_plugin_builder_t* builder, const char* version);
klyron_plugin_config_t* klyron_plugin_builder_build(klyron_plugin_builder_t* builder);

#endif /* KLYRON_PLUGIN_BUILDER_H */
