#ifndef KLYRON_POSTGRES_BINDINGS_FFI_HPP
#define KLYRON_POSTGRES_BINDINGS_FFI_HPP

#include "types.hpp"
#include <functional>

namespace klyron {

extern "C" {
  int klyron_postgres_init();
  const char* klyron_postgres_version();
  char* klyron_postgres_process(const char* input);
  void klyron_postgres_free_string(char* s);
}

} // namespace klyron

#endif
