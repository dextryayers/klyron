#ifndef KLYRON_UPDATER_BINDINGS_API_HPP
#define KLYRON_UPDATER_BINDINGS_API_HPP

#include "types.hpp"
#include "config.hpp"

namespace klyron {

class Klyron::UpdaterApi {
public:
  Klyron::UpdaterApi();
  ~Klyron::UpdaterApi();

  Klyron::UpdaterResult process(const std::string& input);
  std::string version() const;
  bool ping();

private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

} // namespace klyron

#endif
