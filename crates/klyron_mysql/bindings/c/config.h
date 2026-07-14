#ifndef KLYRON_MYSQL_BINDINGS_CONFIG_H
#define KLYRON_MYSQL_BINDINGS_CONFIG_H

#include "types.h"

typedef struct klyron_mysql_settings_t {
  int max_retries;
  long timeout_ms;
  char* log_level;
} klyron_mysql_settings_t;

void klyron_mysql_config_init(klyron_mysql_config_t* config);
klyron_mysql_settings_t klyron_mysql_settings_default(void);

#endif
