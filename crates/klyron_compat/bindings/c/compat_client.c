#include "compat_client.h"
#include <stdlib.h>

struct klyron_compat_client {
    const char* version;
};

klyron_compat_client_t* klyron_compat_client_new(const klyron_compat_config_t* config) {
    (void)config;
    klyron_compat_client_t* c = malloc(sizeof(klyron_compat_client_t));
    if (c) c->version = "0.1.0";
    return c;
}

void klyron_compat_client_free(klyron_compat_client_t* client) {
    free(client);
}

int klyron_compat_client_execute(klyron_compat_client_t* client) {
    (void)client;
    return 0;
}
