#include "config_client.h"
#include <stdlib.h>

struct klyron_config_client {
    const char* version;
};

klyron_config_client_t* klyron_config_client_new(const klyron_config_config_t* config) {
    (void)config;
    klyron_config_client_t* c = malloc(sizeof(klyron_config_client_t));
    if (c) c->version = "0.1.0";
    return c;
}

void klyron_config_client_free(klyron_config_client_t* client) {
    free(client);
}

int klyron_config_client_execute(klyron_config_client_t* client) {
    (void)client;
    return 0;
}
