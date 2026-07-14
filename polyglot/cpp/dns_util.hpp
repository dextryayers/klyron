#ifndef KLYRON_DNS_UTIL_HPP
#define KLYRON_DNS_UTIL_HPP

#include "klyron.hpp"
#include <netdb.h>
#include <arpa/inet.h>
#include <cstring>
#include <sys/socket.h>
#include <unistd.h>

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

    static Vec<DnsRecord> resolve_mx(const String &hostname) {
        Vec<DnsRecord> records;
        // Use system 'host' or 'dig' for MX lookup
        auto r = Process::exec("host -t MX " + hostname + " 2>/dev/null || dig MX " + hostname + " +short 2>/dev/null");
        if (r.success) {
            std::istringstream stream(r.stdout_data);
            String line;
            while (std::getline(stream, line)) {
                if (!line.empty()) {
                    records.push_back({hostname, "MX", line, 0});
                }
            }
        }
        return records;
    }

    static bool is_reachable(const String &host, int port, int timeout_sec = 3) {
        int sock = socket(AF_INET, SOCK_STREAM, 0);
        if (sock < 0) return false;

        struct hostent *server = gethostbyname(host.c_str());
        if (!server) { close(sock); return false; }

        struct sockaddr_in addr;
        std::memset(&addr, 0, sizeof(addr));
        addr.sin_family = AF_INET;
        std::memcpy(&addr.sin_addr.s_addr, server->h_addr, server->h_length);
        addr.sin_port = htons(port);

        struct timeval tv;
        tv.tv_sec = timeout_sec;
        tv.tv_usec = 0;
        setsockopt(sock, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
        setsockopt(sock, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv));

        bool reachable = (connect(sock, (struct sockaddr *)&addr, sizeof(addr)) == 0);
        close(sock);
        return reachable;
    }

    static String reverse_lookup(const String &ip) {
        struct sockaddr_in addr;
        std::memset(&addr, 0, sizeof(addr));
        addr.sin_family = AF_INET;
        inet_pton(AF_INET, ip.c_str(), &addr.sin_addr);

        char host[NI_MAXHOST];
        int res = getnameinfo((struct sockaddr *)&addr, sizeof(addr), host, sizeof(host), nullptr, 0, 0);
        if (res != 0) return "";
        return host;
    }
};

} // namespace klyron

#endif
