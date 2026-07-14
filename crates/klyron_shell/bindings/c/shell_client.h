#ifndef KLYRON_SHELL_CLIENT_H
#define KLYRON_SHELL_CLIENT_H

#include "shell.h"

typedef struct klyron_shell_client klyron_shell_client_t;

klyron_shell_client_t* klyron_shell_client_new(const klyron_shell_config_t* config);
void klyron_shell_client_free(klyron_shell_client_t* client);
int klyron_shell_client_execute(klyron_shell_client_t* client);

#endif /* KLYRON_SHELL_CLIENT_H */
