#include "plugin_client.h"
#include <stdlib.h>

struct klyron_plugin_client {
    const char* version;
};

klyron_plugin_client_t* klyron_plugin_client_new(const klyron_plugin_config_t* config) {
    (void)config;
    klyron_plugin_client_t* c = malloc(sizeof(klyron_plugin_client_t));
    if (c) c->version = "0.1.0";
    return c;
}

void klyron_plugin_client_free(klyron_plugin_client_t* client) {
    free(client);
}

int klyron_plugin_client_execute(klyron_plugin_client_t* client) {
    (void)client;
    return 0;
}
