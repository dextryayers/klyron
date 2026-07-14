#ifndef KLYRON_DOCKER_BUILDER_H
#define KLYRON_DOCKER_BUILDER_H

#include "docker.h"

typedef struct klyron_docker_builder klyron_docker_builder_t;

klyron_docker_builder_t* klyron_docker_builder_new(void);
void klyron_docker_builder_free(klyron_docker_builder_t* builder);
void klyron_docker_builder_set_version(klyron_docker_builder_t* builder, const char* version);
klyron_docker_config_t* klyron_docker_builder_build(klyron_docker_builder_t* builder);

#endif /* KLYRON_DOCKER_BUILDER_H */
