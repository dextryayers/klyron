#ifndef KLYRON_TELEMETRY_CLIENT_H
#define KLYRON_TELEMETRY_CLIENT_H

#include "telemetry.h"

typedef struct klyron_telemetry_client klyron_telemetry_client_t;

klyron_telemetry_client_t* klyron_telemetry_client_new(const klyron_telemetry_config_t* config);
void klyron_telemetry_client_free(klyron_telemetry_client_t* client);
int klyron_telemetry_client_execute(klyron_telemetry_client_t* client);

#endif /* KLYRON_TELEMETRY_CLIENT_H */
