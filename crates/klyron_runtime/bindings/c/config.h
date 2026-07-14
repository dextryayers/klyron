#ifndef KLYRON_RUNTIME_BINDINGS_CONFIG_H
#define KLYRON_RUNTIME_BINDINGS_CONFIG_H

#include "types.h"

typedef struct klyron_runtime_settings_t {
  int max_retries;
  long timeout_ms;
  char* log_level;
} klyron_runtime_settings_t;

void klyron_runtime_config_init(klyron_runtime_config_t* config);
klyron_runtime_settings_t klyron_runtime_settings_default(void);

#endif
