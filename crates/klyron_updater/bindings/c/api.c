#include "api.h"
#include <stdlib.h>
#include <string.h>

klyron_updater_result_t* klyron_updater_process(const char* input) {
  klyron_updater_result_t* result = (klyron_updater_result_t*)malloc(sizeof(klyron_updater_result_t));
  if (!result) return NULL;
  result->success = true;
  result->data = strdup("processed");
  result->error = NULL;
  return result;
}

const char* klyron_updater_version(void) {
  return "klyron_updater 0.1.0";
}

bool klyron_updater_ping(void) {
  return true;
}

void klyron_updater_result_free(klyron_updater_result_t* result) {
  if (result) {
    free(result->data);
    free(result->error);
    free(result);
  }
}
