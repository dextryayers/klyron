#ifndef KLYRON_POSTGRES_BINDINGS_CONFIG_H
#define KLYRON_POSTGRES_BINDINGS_CONFIG_H

#include "types.h"

typedef struct klyron_postgres_settings_t {
  int max_retries;
  long timeout_ms;
  char* log_level;
} klyron_postgres_settings_t;

void klyron_postgres_config_init(klyron_postgres_config_t* config);
klyron_postgres_settings_t klyron_postgres_settings_default(void);

#endif
