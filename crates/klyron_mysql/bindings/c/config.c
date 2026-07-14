#include "config.h"
#include <stdlib.h>
#include <string.h>

void klyron_mysql_config_init(klyron_mysql_config_t* config) {
  if (config) {
    config->enabled = true;
  }
}

klyron_mysql_settings_t klyron_mysql_settings_default(void) {
  klyron_mysql_settings_t s;
  s.max_retries = 3;
  s.timeout_ms = 5000;
  s.log_level = strdup("info");
  return s;
}
