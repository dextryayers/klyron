#ifndef KLYRON_ADAPTER_BUILDER_H
#define KLYRON_ADAPTER_BUILDER_H

#include "adapter.h"

typedef struct klyron_adapter_builder klyron_adapter_builder_t;

klyron_adapter_builder_t* klyron_adapter_builder_new(void);
void klyron_adapter_builder_free(klyron_adapter_builder_t* builder);
void klyron_adapter_builder_set_version(klyron_adapter_builder_t* builder, const char* version);
klyron_adapter_config_t* klyron_adapter_builder_build(klyron_adapter_builder_t* builder);

#endif /* KLYRON_ADAPTER_BUILDER_H */
