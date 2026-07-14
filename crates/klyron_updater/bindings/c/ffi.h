#ifndef KLYRON_UPDATER_BINDINGS_FFI_H
#define KLYRON_UPDATER_BINDINGS_FFI_H

#include "types.h"

int klyron_updater_ffi_init(void);
const char* klyron_updater_ffi_version(void);
char* klyron_updater_ffi_process(const char* input);
void klyron_updater_ffi_free_string(char* s);

#endif
