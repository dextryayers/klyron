#ifndef KLYRON_TEMPLATE_CLIENT_H
#define KLYRON_TEMPLATE_CLIENT_H

#include "template.h"

typedef struct klyron_template_client klyron_template_client_t;

klyron_template_client_t* klyron_template_client_new(const klyron_template_config_t* config);
void klyron_template_client_free(klyron_template_client_t* client);
int klyron_template_client_execute(klyron_template_client_t* client);

#endif /* KLYRON_TEMPLATE_CLIENT_H */
