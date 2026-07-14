#ifndef KLYRON_STRING_H
#define KLYRON_STRING_H

#include "klyron_types.h"

klyron_string_t klyron_string_new(const char *s);
void klyron_string_free(klyron_string_t *s);
klyron_string_t klyron_string_dup(const klyron_string_t *s);
void klyron_string_append(klyron_string_t *dst, const char *src);
klyron_string_t klyron_string_substring(klyron_string_t *s, size_t start, size_t len);
klyron_string_t klyron_string_trim(klyron_string_t *s);
klyron_string_t *klyron_string_split(klyron_string_t *s, const char *delim, size_t *count);
void klyron_string_trim_left(klyron_string_t *s);
void klyron_string_trim_right(klyron_string_t *s);
bool klyron_string_contains(klyron_string_t *s, const char *sub);
bool klyron_string_starts_with(klyron_string_t *s, const char *prefix);
bool klyron_string_ends_with(klyron_string_t *s, const char *suffix);
void klyron_string_replace(klyron_string_t *s, const char *old, const char *new_);
void klyron_string_clear(klyron_string_t *s);
klyron_string_t klyron_string_from_bytes(const uint8_t *bytes, size_t len);

#endif
