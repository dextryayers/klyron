#ifndef KLYRON_TELEMETRY_H
#define KLYRON_TELEMETRY_H

typedef struct {
    const char* version;
} klyron_telemetry_config_t;

klyron_telemetry_config_t* klyron_telemetry_config_new(void);
void klyron_telemetry_config_free(klyron_telemetry_config_t* config);
const char* klyron_telemetry_version(void);

#endif /* KLYRON_TELEMETRY_H */
