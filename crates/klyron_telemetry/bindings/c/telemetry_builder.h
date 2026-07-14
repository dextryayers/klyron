#ifndef KLYRON_TELEMETRY_BUILDER_H
#define KLYRON_TELEMETRY_BUILDER_H

#include "telemetry.h"

typedef struct klyron_telemetry_builder klyron_telemetry_builder_t;

klyron_telemetry_builder_t* klyron_telemetry_builder_new(void);
void klyron_telemetry_builder_free(klyron_telemetry_builder_t* builder);
void klyron_telemetry_builder_set_version(klyron_telemetry_builder_t* builder, const char* version);
klyron_telemetry_config_t* klyron_telemetry_builder_build(klyron_telemetry_builder_t* builder);

#endif /* KLYRON_TELEMETRY_BUILDER_H */
