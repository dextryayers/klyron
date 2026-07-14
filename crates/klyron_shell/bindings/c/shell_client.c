#include "shell_client.h"
#include <stdlib.h>

struct klyron_shell_client {
    const char* version;
};

klyron_shell_client_t* klyron_shell_client_new(const klyron_shell_config_t* config) {
    (void)config;
    klyron_shell_client_t* c = malloc(sizeof(klyron_shell_client_t));
    if (c) c->version = "0.1.0";
    return c;
}

void klyron_shell_client_free(klyron_shell_client_t* client) {
    free(client);
}

int klyron_shell_client_execute(klyron_shell_client_t* client) {
    (void)client;
    return 0;
}
