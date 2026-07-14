#ifndef KLYRON_MYSQL_BINDINGS_TYPES_H
#define KLYRON_MYSQL_BINDINGS_TYPES_H

#include <stdbool.h>
#include <stdint.h>

typedef struct klyron_mysql_config_t {
  bool enabled;
} klyron_mysql_config_t;

typedef struct klyron_mysql_result_t {
  bool success;
  char* data;
  char* error;
} klyron_mysql_result_t;

typedef enum klyron_mysql_status_t {
  STATUS_ACTIVE,
  STATUS_INACTIVE,
  STATUS_ERROR
} klyron_mysql_status_t;

#endif
