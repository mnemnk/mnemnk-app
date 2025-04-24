use anyhow::Result;

use super::agent::Agent;
use super::data::AgentData;

pub trait AgentContext {
    fn try_output(&self, ch: String, data: AgentData) -> Result<()>;

    fn emit_display(&self, key: String, data: AgentData) -> Result<()>;

    #[allow(unused)]
    fn emit_error(&self, message: String) -> Result<()>;
}

impl<T: Agent> AgentContext for T {
    fn try_output(&self, ch: String, data: AgentData) -> Result<()> {
        self.env()
            .try_send_agent_out(self.id().to_string(), ch, data)
    }

    fn emit_display(&self, key: String, data: AgentData) -> Result<()> {
        self.env().emit_display(self.id().to_string(), key, data)
    }

    fn emit_error(&self, message: String) -> Result<()> {
        self.env().emit_error(self.id().to_string(), message)
    }
}
