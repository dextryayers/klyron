#include "deploy_client.h"
#include <stdlib.h>

struct klyron_deploy_client {
    const char* version;
};

klyron_deploy_client_t* klyron_deploy_client_new(const klyron_deploy_config_t* config) {
    (void)config;
    klyron_deploy_client_t* c = malloc(sizeof(klyron_deploy_client_t));
    if (c) c->version = "0.1.0";
    return c;
}

void klyron_deploy_client_free(klyron_deploy_client_t* client) {
    free(client);
}

int klyron_deploy_client_execute(klyron_deploy_client_t* client) {
    (void)client;
    return 0;
}
