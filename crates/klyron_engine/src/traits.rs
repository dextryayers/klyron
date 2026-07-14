use crate::process::EngineOutput;

pub trait EngineTrait {
    fn exec(&mut self, code: &str) -> anyhow::Result<EngineOutput>;
}
