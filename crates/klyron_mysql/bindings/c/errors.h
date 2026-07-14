#ifndef KLYRON_MYSQL_BINDINGS_ERRORS_H
#define KLYRON_MYSQL_BINDINGS_ERRORS_H

typedef enum klyron_mysql_error_code_t {
  ERROR_NONE = 0,
  ERROR_NOT_FOUND,
  ERROR_INVALID_INPUT,
  ERROR_OPERATION_FAILED
} klyron_mysql_error_code_t;

const char* klyron_mysql_error_message(klyron_mysql_error_code_t code);

#endif
