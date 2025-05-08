use anyhow::{bail, Context as _, Result};
use std::fs;
use std::path::Path;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    AgentConfig, AgentContext, AgentData, AgentDefinition, AgentDefinitions, AgentOutput,
    AgentValue, AsAgent, AsAgentData,
};

// List Files Agent
struct ListFilesAgent {
    data: AsAgentData,
}

impl AsAgent for ListFilesAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        let path = data.as_str().context("Path is not a string")?;
        let path = Path::new(path);

        if !path.exists() {
            bail!("Path does not exist: {}", path.display());
        }

        if !path.is_dir() {
            bail!("Path is not a directory: {}", path.display());
        }

        let mut files = Vec::new();
        let entries = fs::read_dir(path).context("Failed to read directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            files.push(AgentValue::new_string(file_name));
        }

        let out_data = AgentData::new_array("string", files);
        self.try_output(ctx, CH_FILES, out_data)
            .context("Failed to output files list")
    }
}

// Read Text File Agent
struct ReadTextFileAgent {
    data: AsAgentData,
}

impl AsAgent for ReadTextFileAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        let path = data.as_str().context("Path is not a string")?;
        let path = Path::new(path);

        if !path.exists() {
            bail!("File does not exist: {}", path.display());
        }

        if !path.is_file() {
            bail!("Path is not a file: {}", path.display());
        }

        let content = fs::read_to_string(path).context("Failed to read file contents")?;
        let out_data = AgentData::new_text(content);
        self.try_output(ctx, CH_TEXT, out_data)
            .context("Failed to output file content")
    }
}

// Write Text File Agent
struct WriteTextFileAgent {
    data: AsAgentData,
}

impl AsAgent for WriteTextFileAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        let input = data.as_object().context("Input is not an object")?;

        let path = input
            .get("path")
            .context("Missing 'path' in input")?
            .as_str()
            .context("'path' is not a string")?;

        let text = input
            .get("text")
            .context("Missing 'text' in input")?
            .as_str()
            .context("'text' is not a string")?;

        let path = Path::new(path);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).context("Failed to create parent directories")?;
            }
        }

        fs::write(path, text).context("Failed to write file")?;

        self.try_output(ctx, CH_DATA, data)
            .context("Failed to output result")
    }
}

static CATEGORY: &str = "Core/File";

static CH_PATH: &str = "path";
static CH_FILES: &str = "files";
static CH_TEXT: &str = "text";
static CH_DATA: &str = "data";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // List Files Agent
    defs.insert(
        "$list_files".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$list_files",
            Some(new_boxed::<ListFilesAgent>),
        )
        .with_title("List Files")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_PATH])
        .with_outputs(vec![CH_FILES]),
    );

    // Read Text File Agent
    defs.insert(
        "$read_text_file".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$read_text_file",
            Some(new_boxed::<ReadTextFileAgent>),
        )
        .with_title("Read Text File")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_PATH])
        .with_outputs(vec![CH_TEXT]),
    );

    // Write Text File Agent
    defs.insert(
        "$write_text_file".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$write_text_file",
            Some(new_boxed::<WriteTextFileAgent>),
        )
        .with_title("Write Text File")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_DATA])
        .with_outputs(vec![CH_DATA]),
    );
}
