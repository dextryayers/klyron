#ifndef KLYRON_CONFIG_CLIENT_H
#define KLYRON_CONFIG_CLIENT_H

#include "config.h"

typedef struct klyron_config_client klyron_config_client_t;

klyron_config_client_t* klyron_config_client_new(const klyron_config_config_t* config);
void klyron_config_client_free(klyron_config_client_t* client);
int klyron_config_client_execute(klyron_config_client_t* client);

#endif /* KLYRON_CONFIG_CLIENT_H */
