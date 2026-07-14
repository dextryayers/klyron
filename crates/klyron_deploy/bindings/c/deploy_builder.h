#ifndef KLYRON_DEPLOY_BUILDER_H
#define KLYRON_DEPLOY_BUILDER_H

#include "deploy.h"

typedef struct klyron_deploy_builder klyron_deploy_builder_t;

klyron_deploy_builder_t* klyron_deploy_builder_new(void);
void klyron_deploy_builder_free(klyron_deploy_builder_t* builder);
void klyron_deploy_builder_set_version(klyron_deploy_builder_t* builder, const char* version);
klyron_deploy_config_t* klyron_deploy_builder_build(klyron_deploy_builder_t* builder);

#endif /* KLYRON_DEPLOY_BUILDER_H */
