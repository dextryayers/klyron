#ifndef KLYRON_TEMPLATE_BUILDER_H
#define KLYRON_TEMPLATE_BUILDER_H

#include "template.h"

typedef struct klyron_template_builder klyron_template_builder_t;

klyron_template_builder_t* klyron_template_builder_new(void);
void klyron_template_builder_free(klyron_template_builder_t* builder);
void klyron_template_builder_set_version(klyron_template_builder_t* builder, const char* version);
klyron_template_config_t* klyron_template_builder_build(klyron_template_builder_t* builder);

#endif /* KLYRON_TEMPLATE_BUILDER_H */
