#ifndef KLYRON_UPDATER_BINDINGS_FFI_HPP
#define KLYRON_UPDATER_BINDINGS_FFI_HPP

#include "types.hpp"
#include <functional>

namespace klyron {

extern "C" {
  int klyron_updater_init();
  const char* klyron_updater_version();
  char* klyron_updater_process(const char* input);
  void klyron_updater_free_string(char* s);
}

} // namespace klyron

#endif
