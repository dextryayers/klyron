#ifndef KLYRON_SQLITE_BINDINGS_CONFIG_H
#define KLYRON_SQLITE_BINDINGS_CONFIG_H

#include "types.h"

typedef struct klyron_sqlite_settings_t {
  int max_retries;
  long timeout_ms;
  char* log_level;
} klyron_sqlite_settings_t;

void klyron_sqlite_config_init(klyron_sqlite_config_t* config);
klyron_sqlite_settings_t klyron_sqlite_settings_default(void);

#endif
