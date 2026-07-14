#ifndef KLYRON_ENGINE_BINDINGS_FFI_HPP
#define KLYRON_ENGINE_BINDINGS_FFI_HPP

#include "types.hpp"
#include <functional>

namespace klyron {

extern "C" {
  int klyron_engine_init();
  const char* klyron_engine_version();
  char* klyron_engine_process(const char* input);
  void klyron_engine_free_string(char* s);
}

} // namespace klyron

#endif
