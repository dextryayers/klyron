#ifndef KLYRON_AI_BINDINGS_FFI_HPP
#define KLYRON_AI_BINDINGS_FFI_HPP

#include "types.hpp"
#include <functional>

namespace klyron {

extern "C" {
  int klyron_ai_init();
  const char* klyron_ai_version();
  char* klyron_ai_process(const char* input);
  void klyron_ai_free_string(char* s);
}

} // namespace klyron

#endif
