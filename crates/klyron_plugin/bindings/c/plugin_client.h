#ifndef KLYRON_PLUGIN_CLIENT_H
#define KLYRON_PLUGIN_CLIENT_H

#include "plugin.h"

typedef struct klyron_plugin_client klyron_plugin_client_t;

klyron_plugin_client_t* klyron_plugin_client_new(const klyron_plugin_config_t* config);
void klyron_plugin_client_free(klyron_plugin_client_t* client);
int klyron_plugin_client_execute(klyron_plugin_client_t* client);

#endif /* KLYRON_PLUGIN_CLIENT_H */
