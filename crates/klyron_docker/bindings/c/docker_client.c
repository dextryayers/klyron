#include "docker_client.h"
#include <stdlib.h>

struct klyron_docker_client {
    const char* version;
};

klyron_docker_client_t* klyron_docker_client_new(const klyron_docker_config_t* config) {
    (void)config;
    klyron_docker_client_t* c = malloc(sizeof(klyron_docker_client_t));
    if (c) c->version = "0.1.0";
    return c;
}

void klyron_docker_client_free(klyron_docker_client_t* client) {
    free(client);
}

int klyron_docker_client_execute(klyron_docker_client_t* client) {
    (void)client;
    return 0;
}
