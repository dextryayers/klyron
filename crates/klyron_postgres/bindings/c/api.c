#include "api.h"
#include <stdlib.h>
#include <string.h>

klyron_postgres_result_t* klyron_postgres_process(const char* input) {
  klyron_postgres_result_t* result = (klyron_postgres_result_t*)malloc(sizeof(klyron_postgres_result_t));
  if (!result) return NULL;
  result->success = true;
  result->data = strdup("processed");
  result->error = NULL;
  return result;
}

const char* klyron_postgres_version(void) {
  return "klyron_postgres 0.1.0";
}

bool klyron_postgres_ping(void) {
  return true;
}

void klyron_postgres_result_free(klyron_postgres_result_t* result) {
  if (result) {
    free(result->data);
    free(result->error);
    free(result);
  }
}
