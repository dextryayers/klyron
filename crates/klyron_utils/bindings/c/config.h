#ifndef KLYRON_UTILS_BINDINGS_CONFIG_H
#define KLYRON_UTILS_BINDINGS_CONFIG_H

#include "types.h"

typedef struct klyron_utils_settings_t {
  int max_retries;
  long timeout_ms;
  char* log_level;
} klyron_utils_settings_t;

void klyron_utils_config_init(klyron_utils_config_t* config);
klyron_utils_settings_t klyron_utils_settings_default(void);

#endif
