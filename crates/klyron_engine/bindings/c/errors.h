#ifndef KLYRON_ENGINE_BINDINGS_ERRORS_H
#define KLYRON_ENGINE_BINDINGS_ERRORS_H

typedef enum klyron_engine_error_code_t {
  ERROR_NONE = 0,
  ERROR_NOT_FOUND,
  ERROR_INVALID_INPUT,
  ERROR_OPERATION_FAILED
} klyron_engine_error_code_t;

const char* klyron_engine_error_message(klyron_engine_error_code_t code);

#endif
