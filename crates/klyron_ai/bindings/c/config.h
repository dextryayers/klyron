#ifndef KLYRON_AI_BINDINGS_CONFIG_H
#define KLYRON_AI_BINDINGS_CONFIG_H

#include "types.h"

typedef struct klyron_ai_settings_t {
  int max_retries;
  long timeout_ms;
  char* log_level;
} klyron_ai_settings_t;

void klyron_ai_config_init(klyron_ai_config_t* config);
klyron_ai_settings_t klyron_ai_settings_default(void);

#endif
