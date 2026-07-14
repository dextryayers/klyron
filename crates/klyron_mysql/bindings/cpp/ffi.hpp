#ifndef KLYRON_MYSQL_BINDINGS_FFI_HPP
#define KLYRON_MYSQL_BINDINGS_FFI_HPP

#include "types.hpp"
#include <functional>

namespace klyron {

extern "C" {
  int klyron_mysql_init();
  const char* klyron_mysql_version();
  char* klyron_mysql_process(const char* input);
  void klyron_mysql_free_string(char* s);
}

} // namespace klyron

#endif
