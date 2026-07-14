#ifndef KLYRON_ADAPTER_CLIENT_H
#define KLYRON_ADAPTER_CLIENT_H

#include "adapter.h"

typedef struct klyron_adapter_client klyron_adapter_client_t;

klyron_adapter_client_t* klyron_adapter_client_new(const klyron_adapter_config_t* config);
void klyron_adapter_client_free(klyron_adapter_client_t* client);
int klyron_adapter_client_execute(klyron_adapter_client_t* client);

#endif /* KLYRON_ADAPTER_CLIENT_H */
