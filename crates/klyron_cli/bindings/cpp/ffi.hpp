#ifndef KLYRON_CLI_BINDINGS_FFI_HPP
#define KLYRON_CLI_BINDINGS_FFI_HPP

#include "types.hpp"
#include <functional>

namespace klyron {

extern "C" {
  int klyron_cli_init();
  const char* klyron_cli_version();
  char* klyron_cli_process(const char* input);
  void klyron_cli_free_string(char* s);
}

} // namespace klyron

#endif
