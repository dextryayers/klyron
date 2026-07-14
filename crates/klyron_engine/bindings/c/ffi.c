#include "ffi.h"
#include <stdlib.h>
#include <string.h>

int klyron_engine_ffi_init(void) {
  return 0;
}

const char* klyron_engine_ffi_version(void) {
  return "klyron_engine 0.1.0";
}

char* klyron_engine_ffi_process(const char* input) {
  if (!input) return strdup("error: null input");
  return strdup("ok");
}

void klyron_engine_ffi_free_string(char* s) {
  free(s);
}
