#ifndef KLYRON_STRING_H
#define KLYRON_STRING_H

#include "klyron_types.h"

klyron_string_t klyron_string_new(const char *s);
void klyron_string_free(klyron_string_t *s);
klyron_string_t klyron_string_dup(const klyron_string_t *s);
void klyron_string_append(klyron_string_t *dst, const char *src);

#endif
