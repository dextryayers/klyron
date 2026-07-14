#ifndef KLYRON_DOCKER_CLIENT_H
#define KLYRON_DOCKER_CLIENT_H

#include "docker.h"

typedef struct klyron_docker_client klyron_docker_client_t;

klyron_docker_client_t* klyron_docker_client_new(const klyron_docker_config_t* config);
void klyron_docker_client_free(klyron_docker_client_t* client);
int klyron_docker_client_execute(klyron_docker_client_t* client);

#endif /* KLYRON_DOCKER_CLIENT_H */
