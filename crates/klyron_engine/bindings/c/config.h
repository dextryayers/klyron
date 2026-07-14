#ifndef KLYRON_ENGINE_BINDINGS_CONFIG_H
#define KLYRON_ENGINE_BINDINGS_CONFIG_H

#include "types.h"

typedef struct klyron_engine_settings_t {
  int max_retries;
  long timeout_ms;
  char* log_level;
} klyron_engine_settings_t;

void klyron_engine_config_init(klyron_engine_config_t* config);
klyron_engine_settings_t klyron_engine_settings_default(void);

#endif
