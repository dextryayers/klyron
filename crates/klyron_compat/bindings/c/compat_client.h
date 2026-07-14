#ifndef KLYRON_COMPAT_CLIENT_H
#define KLYRON_COMPAT_CLIENT_H

#include "compat.h"

typedef struct klyron_compat_client klyron_compat_client_t;

klyron_compat_client_t* klyron_compat_client_new(const klyron_compat_config_t* config);
void klyron_compat_client_free(klyron_compat_client_t* client);
int klyron_compat_client_execute(klyron_compat_client_t* client);

#endif /* KLYRON_COMPAT_CLIENT_H */
