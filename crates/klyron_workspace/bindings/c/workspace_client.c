#include "workspace_client.h"
#include <stdlib.h>

struct klyron_workspace_client {
    const char* version;
};

klyron_workspace_client_t* klyron_workspace_client_new(const klyron_workspace_config_t* config) {
    (void)config;
    klyron_workspace_client_t* c = malloc(sizeof(klyron_workspace_client_t));
    if (c) c->version = "0.1.0";
    return c;
}

void klyron_workspace_client_free(klyron_workspace_client_t* client) {
    free(client);
}

int klyron_workspace_client_execute(klyron_workspace_client_t* client) {
    (void)client;
    return 0;
}
