#ifndef KLYRON_SHELL_BUILDER_H
#define KLYRON_SHELL_BUILDER_H

#include "shell.h"

typedef struct klyron_shell_builder klyron_shell_builder_t;

klyron_shell_builder_t* klyron_shell_builder_new(void);
void klyron_shell_builder_free(klyron_shell_builder_t* builder);
void klyron_shell_builder_set_version(klyron_shell_builder_t* builder, const char* version);
klyron_shell_config_t* klyron_shell_builder_build(klyron_shell_builder_t* builder);

#endif /* KLYRON_SHELL_BUILDER_H */
