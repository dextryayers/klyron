#ifndef KLYRON_UTILS_BINDINGS_API_HPP
#define KLYRON_UTILS_BINDINGS_API_HPP

#include "types.hpp"
#include "config.hpp"

namespace klyron {

class Klyron::UtilsApi {
public:
  Klyron::UtilsApi();
  ~Klyron::UtilsApi();

  Klyron::UtilsResult process(const std::string& input);
  std::string version() const;
  bool ping();

private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

} // namespace klyron

#endif
