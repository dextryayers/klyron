import { DockerConfig } from "./types.js";

function benchConfig(): DockerConfig {
  return { version: "0.1.0" };
}

console.log("benchmark ready");
