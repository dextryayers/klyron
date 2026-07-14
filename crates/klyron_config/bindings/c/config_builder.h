#ifndef KLYRON_CONFIG_BUILDER_H
#define KLYRON_CONFIG_BUILDER_H

#include "config.h"

typedef struct klyron_config_builder klyron_config_builder_t;

klyron_config_builder_t* klyron_config_builder_new(void);
void klyron_config_builder_free(klyron_config_builder_t* builder);
void klyron_config_builder_set_version(klyron_config_builder_t* builder, const char* version);
klyron_config_config_t* klyron_config_builder_build(klyron_config_builder_t* builder);

#endif /* KLYRON_CONFIG_BUILDER_H */
