#include "api.h"
#include <stdlib.h>
#include <string.h>

klyron_ai_result_t* klyron_ai_process(const char* input) {
  klyron_ai_result_t* result = (klyron_ai_result_t*)malloc(sizeof(klyron_ai_result_t));
  if (!result) return NULL;
  result->success = true;
  result->data = strdup("processed");
  result->error = NULL;
  return result;
}

const char* klyron_ai_version(void) {
  return "klyron_ai 0.1.0";
}

bool klyron_ai_ping(void) {
  return true;
}

void klyron_ai_result_free(klyron_ai_result_t* result) {
  if (result) {
    free(result->data);
    free(result->error);
    free(result);
  }
}
