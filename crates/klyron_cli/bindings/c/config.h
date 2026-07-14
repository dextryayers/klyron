#ifndef KLYRON_CLI_BINDINGS_CONFIG_H
#define KLYRON_CLI_BINDINGS_CONFIG_H

#include "types.h"

typedef struct klyron_cli_settings_t {
  int max_retries;
  long timeout_ms;
  char* log_level;
} klyron_cli_settings_t;

void klyron_cli_config_init(klyron_cli_config_t* config);
klyron_cli_settings_t klyron_cli_settings_default(void);

#endif
