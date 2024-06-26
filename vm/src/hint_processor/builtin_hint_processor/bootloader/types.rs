use crate::types::errors::cairo_pie_error::CairoPieError;
use crate::Felt252;
use serde::{de, Deserialize, Deserializer};

use std::path::PathBuf;

use crate::serde::deserialize_program::deserialize_and_parse_program;
use crate::types::errors::program_errors::ProgramError;

use crate::types::program::Program;
use crate::vm::runners::cairo_pie::{CairoPie, StrippedProgram};

pub type BootloaderVersion = u64;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct BootloaderConfig {
    pub simple_bootloader_program_hash: Felt252,
    pub supported_cairo_verifier_program_hashes: Vec<Felt252>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct CompositePackedOutput {
    pub outputs: Vec<Felt252>,
    pub subtasks: Vec<PackedOutput>,
}

impl CompositePackedOutput {
    pub fn elements_for_hash(&self) -> &Vec<Felt252> {
        &self.outputs
    }
}

impl Default for CompositePackedOutput {
    fn default() -> Self {
        Self {
            outputs: Default::default(),
            subtasks: Default::default(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum PackedOutput {
    Plain(Vec<Felt252>),
    Composite(CompositePackedOutput),
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct CairoPiePath {
    path: PathBuf,
    pub use_poseidon: bool,
}

impl CairoPiePath {
    pub fn read(&self) -> Result<CairoPie, CairoPieError> {
        let file = std::fs::File::open(&self.path)?;
        let zip = zip::ZipArchive::new(file)?;
        Ok(CairoPie::from_zip_archive(zip)?)
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Task {
    #[serde(deserialize_with = "deserialize_program")]
    Program(Program),
    Pie(CairoPie),
    CairoPiePath(CairoPiePath),
}

fn deserialize_program<'de, D>(deserializer: D) -> Result<Program, D::Error>
where
    D: Deserializer<'de>,
{
    let obj_raw: &str = Deserialize::deserialize(deserializer)?;
    deserialize_and_parse_program(obj_raw.as_bytes(), Some("main")).map_err(de::Error::custom)
}

impl Task {
    pub fn get_program(&self) -> Result<StrippedProgram, ProgramError> {
        // TODO: consider whether declaring a struct similar to StrippedProgram
        //       but that uses a reference to data to avoid copying is worth the effort.
        match self {
            Task::Program(program) => program.get_stripped_program(),
            Task::Pie(cairo_pie) => Ok(cairo_pie.metadata.program.clone()),
            Task::CairoPiePath(pie_path) => Ok(Task::get_program(&Task::Pie(pie_path.read()?))?),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(transparent)]
pub struct TaskSpec {
    pub task: Task,
}

impl TaskSpec {
    pub fn load_task(&self) -> &Task {
        &self.task
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct SimpleBootloaderInput {
    pub fact_topologies_path: Option<PathBuf>,
    pub single_page: bool,
    pub tasks: Vec<TaskSpec>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct BootloaderInput {
    pub simple_bootloader_input: SimpleBootloaderInput,
    pub bootloader_config: BootloaderConfig,
    pub packed_outputs: Vec<PackedOutput>,
}
