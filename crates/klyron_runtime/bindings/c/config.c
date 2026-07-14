#include "config.h"
#include <stdlib.h>
#include <string.h>

void klyron_runtime_config_init(klyron_runtime_config_t* config) {
  if (config) {
    config->enabled = true;
  }
}

klyron_runtime_settings_t klyron_runtime_settings_default(void) {
  klyron_runtime_settings_t s;
  s.max_retries = 3;
  s.timeout_ms = 5000;
  s.log_level = strdup("info");
  return s;
}
