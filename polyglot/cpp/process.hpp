#ifndef KLYRON_PROCESS_HPP
#define KLYRON_PROCESS_HPP

#include "klyron.hpp"
#include <cstdio>
#include <memory>
#include <thread>
#include <chrono>
#include <csignal>

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

    static int spawn(const String &cmd, const Vec<String> &args = {}) {
        String full_cmd = cmd;
        for (const auto &a : args) full_cmd += " " + a;
        int ret = std::system((full_cmd + " &").c_str());
        return ret;
    }

    static bool kill(int pid, int sig = SIGTERM) {
        return ::kill(pid, sig) == 0;
    }

    static Opt<String> which(const String &program) {
        auto r = exec("which " + program);
        if (r.success && !r.stdout_data.empty()) {
            auto nl = r.stdout_data.find('\n');
            return r.stdout_data.substr(0, nl);
        }
        return std::nullopt;
    }

    static ProcessResult exec_with_stdin(const String &cmd, const String &stdin_data) {
        ProcessResult r;
        FILE *pipe = popen(cmd.c_str(), "w");
        if (!pipe) { r.exit_code = -1; r.success = false; return r; }
        fwrite(stdin_data.data(), 1, stdin_data.size(), pipe);
        r.exit_code = pclose(pipe);
        r.success = r.exit_code == 0;
        return r;
    }

    static int pid() {
        return static_cast<int>(::getpid());
    }

    static void sleep_ms(uint64_t ms) {
        std::this_thread::sleep_for(std::chrono::milliseconds(ms));
    }
};

} // namespace klyron

#endif
