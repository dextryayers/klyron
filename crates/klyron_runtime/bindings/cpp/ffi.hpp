#ifndef KLYRON_RUNTIME_BINDINGS_FFI_HPP
#define KLYRON_RUNTIME_BINDINGS_FFI_HPP

#include "types.hpp"
#include <functional>

namespace klyron {

extern "C" {
  int klyron_runtime_init();
  const char* klyron_runtime_version();
  char* klyron_runtime_process(const char* input);
  void klyron_runtime_free_string(char* s);
}

} // namespace klyron

#endif
