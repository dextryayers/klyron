#ifndef KLYRON_PROCESS_HPP
#define KLYRON_PROCESS_HPP

#include "klyron.hpp"
#include <cstdio>
#include <memory>

namespace klyron {

class Process {
public:
    static ProcessResult exec(const String &cmd) {
        ProcessResult r;
        std::array<char, 4096> buf;
        auto deleter = [](FILE *f) { if (f) pclose(f); };
        std::unique_ptr<FILE, decltype(deleter)> pipe(popen(cmd.c_str(), "r"), deleter);
        if (!pipe) { r.exit_code = -1; r.success = false; return r; }
        while (fgets(buf.data(), buf.size(), pipe.get())) {
            r.stdout_data += buf.data();
        }
        r.exit_code = pclose(pipe.release());
        r.success = r.exit_code == 0;
        return r;
    }

    static Opt<String> which(const String &program) {
        auto r = exec("which " + program);
        if (r.success && !r.stdout_data.empty()) {
            auto nl = r.stdout_data.find('\n');
            return r.stdout_data.substr(0, nl);
        }
        return std::nullopt;
    }
};

} // namespace klyron

#endif
