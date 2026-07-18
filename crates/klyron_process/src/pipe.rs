use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};

use crate::ProcessResult;

pub struct PipeBuilder {
    stages: Vec<PipeStage>,
}

struct PipeStage {
    program: String,
    args: Vec<String>,
}

impl PipeBuilder {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn add_stage(mut self, program: &str, args: &[&str]) -> Self {
        self.stages.push(PipeStage {
            program: program.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    pub fn build(self) -> anyhow::Result<Pipeline> {
        if self.stages.is_empty() {
            anyhow::bail!("Empty pipeline");
        }
        Ok(Pipeline { stages: self.stages })
    }

    pub fn execute(self) -> anyhow::Result<ProcessResult> {
        let pipeline = self.build()?;
        pipeline.execute()
    }
}

impl Default for PipeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Pipeline {
    stages: Vec<PipeStage>,
}

impl Pipeline {
    pub fn execute(&self) -> anyhow::Result<ProcessResult> {
        if self.stages.is_empty() {
            anyhow::bail!("Empty pipeline");
        }

        let mut cmds: Vec<Command> = self.stages.iter().map(|s| {
            let mut cmd = Command::new(&s.program);
            cmd.args(&s.args);
            cmd
        }).collect();

        for i in 0..cmds.len() {
            cmds[i].stdout(Stdio::piped());
            if i > 0 {
                cmds[i].stdin(Stdio::piped());
            }
        }
        cmds.last_mut().unwrap().stderr(Stdio::piped());

        let mut children: Vec<Child> = cmds.into_iter()
            .map(|mut c| c.spawn())
            .collect::<Result<Vec<_>, _>>()?;

        for i in 0..children.len().saturating_sub(1) {
            let stdout = children[i].stdout.take()
                .ok_or_else(|| anyhow::anyhow!("no stdout on pipe element {i}"))?;
            let stdin = children[i + 1].stdin.take()
                .ok_or_else(|| anyhow::anyhow!("no stdin on pipe element {}", i + 1))?;
            let mut reader = stdout;
            let mut writer = stdin;
            std::thread::spawn(move || {
                let mut buf = [0u8; 8192];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => { writer.write_all(&buf[..n]).ok(); }
                    }
                }
            });
        }

        let output = children.into_iter().last()
            .ok_or_else(|| anyhow::anyhow!("no children in pipeline"))?
            .wait_with_output()?;
        Ok(output.into())
    }

    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }
}

pub fn piped(pipeline: &[&str]) -> anyhow::Result<ProcessResult> {
    let mut builder = PipeBuilder::new();
    for part in pipeline {
        let parts: Vec<&str> = part.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        builder = builder.add_stage(parts[0], &parts[1..]);
    }
    builder.execute()
}

pub fn exec_piped(cmds: &[(&str, &[&str])]) -> anyhow::Result<ProcessResult> {
    let mut builder = PipeBuilder::new();
    for (prog, args) in cmds {
        builder = builder.add_stage(prog, args);
    }
    builder.execute()
}
