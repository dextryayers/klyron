#ifndef KLYRON_COMPAT_BUILDER_H
#define KLYRON_COMPAT_BUILDER_H

#include "compat.h"

typedef struct klyron_compat_builder klyron_compat_builder_t;

klyron_compat_builder_t* klyron_compat_builder_new(void);
void klyron_compat_builder_free(klyron_compat_builder_t* builder);
void klyron_compat_builder_set_version(klyron_compat_builder_t* builder, const char* version);
klyron_compat_config_t* klyron_compat_builder_build(klyron_compat_builder_t* builder);

#endif /* KLYRON_COMPAT_BUILDER_H */
