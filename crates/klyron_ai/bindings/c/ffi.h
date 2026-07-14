#ifndef KLYRON_AI_BINDINGS_FFI_H
#define KLYRON_AI_BINDINGS_FFI_H

#include "types.h"

int klyron_ai_ffi_init(void);
const char* klyron_ai_ffi_version(void);
char* klyron_ai_ffi_process(const char* input);
void klyron_ai_ffi_free_string(char* s);

#endif
