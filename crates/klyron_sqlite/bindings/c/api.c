#include "api.h"
#include <stdlib.h>
#include <string.h>

klyron_sqlite_result_t* klyron_sqlite_process(const char* input) {
  klyron_sqlite_result_t* result = (klyron_sqlite_result_t*)malloc(sizeof(klyron_sqlite_result_t));
  if (!result) return NULL;
  result->success = true;
  result->data = strdup("processed");
  result->error = NULL;
  return result;
}

const char* klyron_sqlite_version(void) {
  return "klyron_sqlite 0.1.0";
}

bool klyron_sqlite_ping(void) {
  return true;
}

void klyron_sqlite_result_free(klyron_sqlite_result_t* result) {
  if (result) {
    free(result->data);
    free(result->error);
    free(result);
  }
}
