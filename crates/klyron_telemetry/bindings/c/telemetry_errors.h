#ifndef KLYRON_TELEMETRY_ERRORS_H
#define KLYRON_TELEMETRY_ERRORS_H

#define KLYRON_TELEMETRY_OK 0
#define KLYRON_TELEMETRY_ERR_INIT -1
#define KLYRON_TELEMETRY_ERR_OPERATION -2

const char* klyron_telemetry_error_string(int err);

#endif /* KLYRON_TELEMETRY_ERRORS_H */
