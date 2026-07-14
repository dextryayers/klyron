#ifndef KLYRON_KLYRON_DNS_FFI_HPP
#define KLYRON_KLYRON_DNS_FFI_HPP

extern "C" {
    const char* klyron_dns_version();
}

inline const char* klyron_dns_version_str() { return klyron_dns_version(); }

#endif
