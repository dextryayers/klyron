#ifndef KLYRON_WORKSPACE_BUILDER_H
#define KLYRON_WORKSPACE_BUILDER_H

#include "workspace.h"

typedef struct klyron_workspace_builder klyron_workspace_builder_t;

klyron_workspace_builder_t* klyron_workspace_builder_new(void);
void klyron_workspace_builder_free(klyron_workspace_builder_t* builder);
void klyron_workspace_builder_set_version(klyron_workspace_builder_t* builder, const char* version);
klyron_workspace_config_t* klyron_workspace_builder_build(klyron_workspace_builder_t* builder);

#endif /* KLYRON_WORKSPACE_BUILDER_H */
