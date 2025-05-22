#[cfg(feature = "rig")]
mod implementation {
    use anyhow::{bail, Context as _, Result};
    use rig::completion::CompletionRequestBuilder;
    use rig::OneOrMany;
    use std::sync::{Arc, Mutex};
    use tauri::AppHandle;

    use rig::providers::ollama::Client;

    use crate::mnemnk::agent::{
        Agent, AgentConfig, AgentContext, AgentData, AgentOutput, AgentValueMap, AsAgent,
        AsAgentData,
    };

    use super::*;

    // Rig Ollama Agent
    pub struct RigOllamaAgent {
        data: AsAgentData,
        client: Arc<Mutex<Option<Client>>>,
    }

    impl RigOllamaAgent {
        fn get_ollama_url(&self) -> Result<String> {
            let mut ollama_url = self
                .global_config()
                .context("missing global config")?
                .get_string_or_default(CONFIG_OLLAMA_URL);
            if ollama_url.is_empty() {
                if let Ok(ollama_host) = std::env::var("OLLAMA_HOST") {
                    ollama_url = format!("http://{}", ollama_host);
                } else {
                    ollama_url = DEFAULT_OLLAMA_URL.to_string();
                }
            }
            Ok(ollama_url)
        }

        fn get_client(&mut self) -> Result<Client> {
            let mut client_guard = self
                .client
                .lock()
                .map_err(|e| anyhow::anyhow!("Client mutex poisoned: {}", e))?;

            if let Some(client) = client_guard.as_ref() {
                return Ok(client.clone());
            }

            let ollama_url = self.get_ollama_url()?;
            let new_client = Client::from_url(&ollama_url);
            *client_guard = Some(new_client.clone());

            Ok(new_client)
        }
    }

    impl AsAgent for RigOllamaAgent {
        fn new(
            app: AppHandle,
            id: String,
            def_name: String,
            config: Option<AgentConfig>,
        ) -> Result<Self> {
            Ok(Self {
                data: AsAgentData::new(app, id, def_name, config),
                client: Arc::new(Mutex::new(None)),
            })
        }

        fn data(&self) -> &AsAgentData {
            &self.data
        }

        fn mut_data(&mut self) -> &mut AsAgentData {
            &mut self.data
        }

        fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
            let config_model = &self
                .config()
                .context("missing config")?
                .get_string_or_default(CONFIG_MODEL);
            if config_model.is_empty() {
                return Ok(());
            }

            let client = self.get_client()?;
            let comp_model = client.completion_model(config_model);

            let prompts = data_to_prompts(data)?;

            let mut out_messages = Vec::new();
            let mut out_responses = Vec::new();

            for prompt in prompts {
                let comp_model = comp_model.clone();
                let response = tauri::async_runtime::block_on(async move {
                    let user_message = prompt.message;
                    let preamble = prompt.preamble;

                    let mut builder = CompletionRequestBuilder::new(comp_model, user_message);
                    if let Some(preamble) = preamble {
                        builder = builder.preamble(preamble);
                    }
                    builder.send().await
                })?;

                let msg_json = serde_json::to_value(response.raw_response.message.clone())?;
                let msg_value = AgentValue::from_json_value(msg_json)?;
                out_messages.push(msg_value);

                let resp_json = serde_json::to_value(response.raw_response)?;
                let resp_value = AgentValue::from_json_value(resp_json)?;
                out_responses.push(resp_value);
            }

            if out_messages.len() == 1 {
                let out_message = AgentData::new_custom_object(
                    "message",
                    out_messages[0]
                        .as_object()
                        .context("wrong object")?
                        .to_owned(),
                );
                self.try_output(ctx.clone(), CH_MESSAGE, out_message)
                    .context("Failed to output")?;
            } else if out_messages.len() > 1 {
                let out_message = AgentData::new_array("message", out_messages);
                self.try_output(ctx.clone(), CH_MESSAGE, out_message)
                    .context("Failed to output")?;
            }

            if out_responses.len() == 1 {
                let out_response = AgentData::new_custom_object(
                    "response",
                    out_responses[0]
                        .as_object()
                        .context("wrong object")?
                        .to_owned(),
                );
                self.try_output(ctx, CH_RESPONSE, out_response)
                    .context("Failed to output")?;
            } else if out_responses.len() > 1 {
                let out_response = AgentData::new_array("response", out_responses);
                self.try_output(ctx, CH_RESPONSE, out_response)
                    .context("Failed to output")?;
            }

            Ok(())
        }
    }

    struct Prompt {
        message: rig::completion::Message,
        preamble: Option<String>,
    }

    fn data_to_prompts(data: AgentData) -> Result<Vec<Prompt>> {
        let mut prompts = Vec::new();

        if data.is_array() {
            let arr = data.as_array().context("wrong array")?.to_owned();
            for item in arr {
                let preamble = preamble_from_value(&item);
                let user_message = value_to_user_message(item)?;
                prompts.push(Prompt {
                    message: user_message,
                    preamble,
                });
            }
            return Ok(prompts);
        }

        let preamble = preamble_from_value(&data.value);
        let user_message = value_to_user_message(data.value)?;

        prompts.push(Prompt {
            message: user_message,
            preamble,
        });

        Ok(prompts)
    }

    fn preamble_from_value(value: &AgentValue) -> Option<String> {
        if value.is_string() {
            return None;
        }

        if value.is_object() {
            return value.get_str("preamble").map(|s| s.to_string());
        }

        None
    }

    fn value_to_user_message(value: AgentValue) -> Result<rig::completion::Message> {
        if value.is_string() {
            let text = value.as_str().context("wrong string")?;

            return Ok(rig::completion::Message::user(text));
        }

        if value.is_object() {
            let role = value.get_str("role").unwrap_or_default();
            if !(role.is_empty() || role == "user") {
                bail!("role is not user");
            }

            let content = value.get_str("content").or_else(|| value.get_str("text"));

            let mut images: Option<Vec<String>> = None;
            if let Some(image) = value.get("image") {
                if image.is_image() {
                    let image = image.as_image().context("wrong image")?.get_base64();
                    images = Some(vec![image]);
                } else if image.is_string() {
                    let image = image.as_str().context("wrong string")?;
                    images = Some(vec![image.to_string()]);
                } else {
                    bail!("invalid image property");
                }
            } else if let Some(images_value) = value.get("images") {
                if images_value.is_array() {
                    let arr = images_value.as_array().context("wrong array")?;
                    let mut images_vec = Vec::new();
                    for image in arr.iter() {
                        if image.is_image() {
                            let image = image.as_image().context("wrong image")?;
                            images_vec.push(image.get_base64().to_string());
                        } else if image.is_string() {
                            let image = image.as_str().context("wrong string")?;
                            images_vec.push(image.to_string());
                        } else {
                            bail!("invalid images property");
                        }
                    }
                    images = Some(images_vec);
                } else {
                    bail!("invalid images property");
                }
            }

            if content.is_none() && images.is_none() {
                bail!("Both content and images are None");
            }

            let mut items = Vec::new();
            if content.is_some() {
                items.push(rig::completion::message::UserContent::Text(
                    rig::completion::message::Text {
                        text: content.unwrap().to_string(),
                    },
                ));
            }
            if images.is_some() {
                for image in images.unwrap() {
                    items.push(rig::completion::message::UserContent::Image(
                        rig::completion::message::Image {
                            data: image
                                .trim_start_matches("data:image/png;base64,")
                                .to_string(),
                            format: None,
                            media_type: None,
                            detail: None,
                        },
                    ));
                }
            }

            return Ok(rig::completion::Message::User {
                content: OneOrMany::many(items)?,
            });
        };

        bail!("Unsupported data type");
    }

    // Rig Preamble Agent
    pub struct RigPreambleAgent {
        data: AsAgentData,
    }

    impl AsAgent for RigPreambleAgent {
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
            let preamble = self
                .config()
                .context("missing config")?
                .get_string_or_default(CONFIG_TEXT);

            if preamble.is_empty() {
                self.try_output(ctx, CH_MESSAGE, data)
                    .context("Failed to output")?;
                return Ok(());
            }

            let data = add_preamble_to_data(preamble, data)?;

            self.try_output(ctx, CH_MESSAGE, data)
                .context("Failed to output")?;

            Ok(())
        }
    }

    fn add_preamble_to_data(preamble: String, data: AgentData) -> Result<AgentData> {
        let value = add_preamble_to_value(preamble, data.value)?;

        if value.is_object() {
            let map = value.as_object().context("wrong object")?.to_owned();
            return Ok(AgentData::new_custom_object("message", map));
        }

        if value.is_array() {
            let arr = value.as_array().context("wrong array")?.to_owned();
            return Ok(AgentData::new_array("message", arr));
        }

        bail!("Unsupported data type");
    }

    fn add_preamble_to_value(preamble: String, value: AgentValue) -> Result<AgentValue> {
        if value.is_string() {
            let content = value.as_str().context("wrong string")?;
            let mut out_value = AgentValueMap::new();
            out_value.insert("content".to_string(), AgentValue::new_string(content));
            out_value.insert("role".to_string(), AgentValue::new_string("user"));
            out_value.insert("preamble".to_string(), AgentValue::new_string(preamble));
            return Ok(AgentValue::new_object(out_value));
        }

        if value.is_object() {
            let mut out_value = value.as_object().context("wrong object value")?.clone();
            out_value.insert("preamble".to_string(), AgentValue::new_string(preamble));
            return Ok(AgentValue::new_object(out_value));
        }

        if value.is_array() {
            let arr = value.as_array().context("wrong array")?.to_owned();
            let mut out_value = Vec::new();
            for item in arr {
                let item = add_preamble_to_value(preamble.clone(), item)?;
                out_value.push(item);
            }
            return Ok(AgentValue::new_array(out_value));
        }

        bail!("Unsupported value type");
    }

    // Rig User Message with Image Agent
    pub struct RigUserMessageWithImageAgent {
        data: AsAgentData,
    }

    impl AsAgent for RigUserMessageWithImageAgent {
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
            let text = self
                .config()
                .context("missing config")?
                .get_string_or_default(CONFIG_TEXT);

            let out_data = combine_text_and_image_data(text, data)?;

            self.try_output(ctx, CH_MESSAGE, out_data)
                .context("Failed to output")?;

            Ok(())
        }
    }

    fn combine_text_and_image_data(text: String, data: AgentData) -> Result<AgentData> {
        let value = combine_text_and_image_value(text, data.value)?;

        if value.is_object() {
            let map = value.as_object().context("wrong object")?.to_owned();
            return Ok(AgentData::new_custom_object("message", map));
        }

        if value.is_array() {
            let arr = value.as_array().context("wrong array")?.to_owned();
            return Ok(AgentData::new_array("message", arr));
        }

        bail!("Unsupported data type");
    }

    fn combine_text_and_image_value(text: String, value: AgentValue) -> Result<AgentValue> {
        if value.is_image() || value.is_string() {
            let mut out_value = AgentValueMap::new();
            out_value.insert("images".to_string(), AgentValue::new_array(vec![value]));
            out_value.insert("role".to_string(), AgentValue::new_string("user"));
            out_value.insert("content".to_string(), AgentValue::new_string(text));
            return Ok(AgentValue::new_object(out_value));
        }

        if value.is_object() {
            let mut out_value = value.as_object().context("wrong object value")?.clone();
            if let Some(images) = value.get("images") {
                if images.is_array() {
                    let images = images.as_array().context("wrong array")?.clone();
                    out_value.insert("images".to_string(), AgentValue::new_array(images));
                } else {
                    bail!("images is not an array");
                }
            } else if let Some(image) = value.get("image") {
                if image.is_image() {
                    out_value.insert(
                        "images".to_string(),
                        AgentValue::new_array(vec![image.clone()]),
                    );
                } else {
                    bail!("image is not an image");
                }
            } else {
                bail!("image or images are not set");
            }
            out_value.insert("role".to_string(), AgentValue::new_string("user"));
            out_value.insert("content".to_string(), AgentValue::new_string(text));
            return Ok(AgentValue::new_object(out_value));
        }

        if value.is_array() {
            let arr = value.as_array().context("wrong array")?.to_owned();
            let mut out_value = Vec::new();
            for item in arr {
                let item = combine_text_and_image_value(text.clone(), item)?;
                out_value.push(item);
            }
            return Ok(AgentValue::new_array(out_value));
        }

        bail!("Unsupported value type");
    }
}

#[cfg(not(feature = "rig"))]
mod implementation {}

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{AgentConfigEntry, AgentDefinition, AgentDefinitions, AgentValue};

static CATEGORY: &str = "Core/Rig";

static CH_IMAGE: &str = "image";
static CH_MESSAGE: &str = "message";
static CH_RESPONSE: &str = "response";

static CONFIG_MODEL: &str = "model";
static CONFIG_OLLAMA_URL: &str = "ollama_url";
static CONFIG_TEXT: &str = "prompt";

static DEFAULT_CONFIG_MODEL: &str = "gemma3:4b";
static DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    #[cfg(feature = "rig")]
    {
        use implementation::*;

        defs.insert(
            "$rig_ollama".to_string(),
            AgentDefinition::new(
                AGENT_KIND_BUILTIN,
                "$rig_ollama",
                Some(new_boxed::<RigOllamaAgent>),
            )
            .use_native_thread()
            .with_title("Rig Ollama")
            .with_category(CATEGORY)
            .with_inputs(vec![CH_MESSAGE])
            .with_outputs(vec![CH_MESSAGE, CH_RESPONSE])
            .with_global_config(vec![(
                CONFIG_OLLAMA_URL.into(),
                AgentConfigEntry::new(AgentValue::new_string(DEFAULT_OLLAMA_URL), "string")
                    .with_title("Ollama URL"),
            )])
            .with_default_config(vec![(
                CONFIG_MODEL.into(),
                AgentConfigEntry::new(AgentValue::new_string(DEFAULT_CONFIG_MODEL), "string")
                    .with_title("Chat Model"),
            )]),
        );

        defs.insert(
            "$rig_preamble".to_string(),
            AgentDefinition::new(
                AGENT_KIND_BUILTIN,
                "$rig_preamble",
                Some(new_boxed::<RigPreambleAgent>),
            )
            .use_native_thread()
            .with_title("Rig Preamble")
            .with_category(CATEGORY)
            .with_inputs(vec![CH_MESSAGE])
            .with_outputs(vec![CH_MESSAGE])
            .with_default_config(vec![(
                CONFIG_TEXT.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "text"),
            )]),
        );

        defs.insert(
            "$rig_user_message_with_image".to_string(),
            AgentDefinition::new(
                AGENT_KIND_BUILTIN,
                "$rig_user_message_with_image",
                Some(new_boxed::<RigUserMessageWithImageAgent>),
            )
            .use_native_thread()
            .with_title("Rig User Message with Image")
            .with_category(CATEGORY)
            .with_inputs(vec![CH_IMAGE])
            .with_outputs(vec![CH_MESSAGE])
            .with_default_config(vec![(
                CONFIG_TEXT.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "text"),
            )]),
        );
    }
}
