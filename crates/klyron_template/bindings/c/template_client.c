#include "template_client.h"
#include <stdlib.h>

struct klyron_template_client {
    const char* version;
};

klyron_template_client_t* klyron_template_client_new(const klyron_template_config_t* config) {
    (void)config;
    klyron_template_client_t* c = malloc(sizeof(klyron_template_client_t));
    if (c) c->version = "0.1.0";
    return c;
}

void klyron_template_client_free(klyron_template_client_t* client) {
    free(client);
}

int klyron_template_client_execute(klyron_template_client_t* client) {
    (void)client;
    return 0;
}
