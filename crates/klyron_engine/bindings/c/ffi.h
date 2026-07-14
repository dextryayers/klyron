#ifndef KLYRON_ENGINE_BINDINGS_FFI_H
#define KLYRON_ENGINE_BINDINGS_FFI_H

#include "types.h"

int klyron_engine_ffi_init(void);
const char* klyron_engine_ffi_version(void);
char* klyron_engine_ffi_process(const char* input);
void klyron_engine_ffi_free_string(char* s);

#endif
