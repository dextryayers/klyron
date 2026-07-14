#include "telemetry_client.h"
#include <stdlib.h>

struct klyron_telemetry_client {
    const char* version;
};

klyron_telemetry_client_t* klyron_telemetry_client_new(const klyron_telemetry_config_t* config) {
    (void)config;
    klyron_telemetry_client_t* c = malloc(sizeof(klyron_telemetry_client_t));
    if (c) c->version = "0.1.0";
    return c;
}

void klyron_telemetry_client_free(klyron_telemetry_client_t* client) {
    free(client);
}

int klyron_telemetry_client_execute(klyron_telemetry_client_t* client) {
    (void)client;
    return 0;
}
