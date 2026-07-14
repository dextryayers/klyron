#include "adapter_client.h"
#include <stdlib.h>

struct klyron_adapter_client {
    const char* version;
};

klyron_adapter_client_t* klyron_adapter_client_new(const klyron_adapter_config_t* config) {
    (void)config;
    klyron_adapter_client_t* c = malloc(sizeof(klyron_adapter_client_t));
    if (c) c->version = "0.1.0";
    return c;
}

void klyron_adapter_client_free(klyron_adapter_client_t* client) {
    free(client);
}

int klyron_adapter_client_execute(klyron_adapter_client_t* client) {
    (void)client;
    return 0;
}
