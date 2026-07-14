#ifndef KLYRON_HTTP_H
#define KLYRON_HTTP_H

#include "klyron_types.h"

klyron_response_t *klyron_http_get(const char *url);
klyron_response_t *klyron_http_post(const char *url, const char *body, const char *content_type);
void klyron_http_free_response(klyron_response_t *resp);

#endif
