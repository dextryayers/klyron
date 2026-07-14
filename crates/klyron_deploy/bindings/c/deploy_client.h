#ifndef KLYRON_DEPLOY_CLIENT_H
#define KLYRON_DEPLOY_CLIENT_H

#include "deploy.h"

typedef struct klyron_deploy_client klyron_deploy_client_t;

klyron_deploy_client_t* klyron_deploy_client_new(const klyron_deploy_config_t* config);
void klyron_deploy_client_free(klyron_deploy_client_t* client);
int klyron_deploy_client_execute(klyron_deploy_client_t* client);

#endif /* KLYRON_DEPLOY_CLIENT_H */
