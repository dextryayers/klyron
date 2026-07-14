#ifndef KLYRON_SQLITE_BINDINGS_FFI_HPP
#define KLYRON_SQLITE_BINDINGS_FFI_HPP

#include "types.hpp"
#include <functional>

namespace klyron {

extern "C" {
  int klyron_sqlite_init();
  const char* klyron_sqlite_version();
  char* klyron_sqlite_process(const char* input);
  void klyron_sqlite_free_string(char* s);
}

} // namespace klyron

#endif
