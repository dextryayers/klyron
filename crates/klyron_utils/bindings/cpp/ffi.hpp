#ifndef KLYRON_UTILS_BINDINGS_FFI_HPP
#define KLYRON_UTILS_BINDINGS_FFI_HPP

#include "types.hpp"
#include <functional>

namespace klyron {

extern "C" {
  int klyron_utils_init();
  const char* klyron_utils_version();
  char* klyron_utils_process(const char* input);
  void klyron_utils_free_string(char* s);
}

} // namespace klyron

#endif
