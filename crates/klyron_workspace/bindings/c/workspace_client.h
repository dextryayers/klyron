#ifndef KLYRON_WORKSPACE_CLIENT_H
#define KLYRON_WORKSPACE_CLIENT_H

#include "workspace.h"

typedef struct klyron_workspace_client klyron_workspace_client_t;

klyron_workspace_client_t* klyron_workspace_client_new(const klyron_workspace_config_t* config);
void klyron_workspace_client_free(klyron_workspace_client_t* client);
int klyron_workspace_client_execute(klyron_workspace_client_t* client);

#endif /* KLYRON_WORKSPACE_CLIENT_H */
