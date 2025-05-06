use anyhow::Result;

use super::agent::Agent;
use super::data::AgentData;

pub trait AgentOutput {
    fn try_output_raw(&self, ch: String, data: AgentData) -> Result<()>;

    fn try_output<S: Into<String>>(&self, ch: S, data: AgentData) -> Result<()> {
        self.try_output_raw(ch.into(), data)
    }

    fn emit_display_raw(&self, key: String, data: AgentData) -> Result<()>;

    fn emit_display<S: Into<String>>(&self, key: S, data: AgentData) -> Result<()> {
        self.emit_display_raw(key.into(), data)
    }

    fn emit_error_raw(&self, message: String) -> Result<()>;

    #[allow(unused)]
    fn emit_error<S: Into<String>>(&self, message: S) -> Result<()> {
        self.emit_error_raw(message.into())
    }
}

impl<T: Agent> AgentOutput for T {
    fn try_output_raw(&self, ch: String, data: AgentData) -> Result<()> {
        self.env()
            .try_send_agent_out(self.id().to_string(), ch, data)
    }

    fn emit_display_raw(&self, key: String, data: AgentData) -> Result<()> {
        self.env().emit_display(self.id().to_string(), key, data)
    }

    fn emit_error_raw(&self, message: String) -> Result<()> {
        self.env().emit_error(self.id().to_string(), message)
    }
}
