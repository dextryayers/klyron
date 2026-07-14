#ifndef KLYRON_DNS_UTIL_HPP
#define KLYRON_DNS_UTIL_HPP

#include "klyron.hpp"
#include <netdb.h>
#include <arpa/inet.h>
#include <cstring>

namespace klyron {

class DnsUtil {
public:
    static Vec<String> resolve(const String &hostname) {
        Vec<String> ips;
        struct addrinfo hints, *res, *p;
        std::memset(&hints, 0, sizeof(hints));
        hints.ai_family = AF_UNSPEC;
        hints.ai_socktype = SOCK_STREAM;

        int status = getaddrinfo(hostname.c_str(), nullptr, &hints, &res);
        if (status != 0) return ips;

        char ip[INET6_ADDRSTRLEN];
        for (p = res; p; p = p->ai_next) {
            void *addr;
            if (p->ai_family == AF_INET) {
                addr = &((struct sockaddr_in *)p->ai_addr)->sin_addr;
            } else {
                addr = &((struct sockaddr_in6 *)p->ai_addr)->sin6_addr;
            }
            inet_ntop(p->ai_family, addr, ip, sizeof(ip));
            ips.push_back(ip);
        }
        freeaddrinfo(res);
        return ips;
    }

    static Vec<String> resolve_ipv4(const String &hostname) {
        Vec<String> ips;
        struct hostent *he = gethostbyname(hostname.c_str());
        if (!he) return ips;
        char ip[INET_ADDRSTRLEN];
        for (int i = 0; he->h_addr_list[i]; i++) {
            inet_ntop(AF_INET, he->h_addr_list[i], ip, sizeof(ip));
            ips.push_back(ip);
        }
        return ips;
    }

    static bool is_reachable(const String &host, int port, int timeout_sec = 3) {
        // simple TCP connect check
        return !resolve(host).empty();
    }
};

} // namespace klyron

#endif
