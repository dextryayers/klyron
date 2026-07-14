#ifndef KLYRON_ENGINE_BINDINGS_API_HPP
#define KLYRON_ENGINE_BINDINGS_API_HPP

#include "types.hpp"
#include "config.hpp"

namespace klyron {

class Klyron::EngineApi {
public:
  Klyron::EngineApi();
  ~Klyron::EngineApi();

  Klyron::EngineResult process(const std::string& input);
  std::string version() const;
  bool ping();

private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

} // namespace klyron

#endif
