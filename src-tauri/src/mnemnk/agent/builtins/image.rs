use anyhow::{Context as _, Result};
use photon_rs::PhotonImage;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentOutput, AgentValue, AsAgent, AsAgentData,
};

// Image Crop
struct ImageCropAgent {
    data: AsAgentData,
    bounding: Option<AgentData>,
}

impl AsAgent for ImageCropAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            bounding: None,
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        if ctx.ch() == CH_BOUNDING {
            if data.get_i64("x").is_none()
                || data.get_i64("y").is_none()
                || data.get_i64("width").is_none()
                || data.get_i64("height").is_none()
            {
                self.bounding = None;
            } else {
                self.bounding = Some(data);
            }
            return Ok(());
        }

        if ctx.ch() == CH_IMAGE {
            if let Some(bounding) = &self.bounding {
                let image: Option<&PhotonImage> = if data.is_image() {
                    data.as_image()
                } else {
                    data.get_image("image")
                };

                let Some(image) = image else {
                    return Ok(());
                };

                let x = bounding.get_i64("x").context("missing x")? as u32;
                let y = bounding.get_i64("y").context("missing y")? as u32;
                let width = bounding.get_i64("width").context("missing width")? as u32;
                let height = bounding.get_i64("height").context("missing height")? as u32;
                let x2 = x + width;
                let y2 = y + height;

                // Check if the bounding box is within the image bounds
                let image_width = image.get_width();
                let image_height = image.get_height();
                if width > 0
                    && height > 0
                    && x < image_width
                    && y < image_height
                    && x2 <= image_width
                    && y2 <= image_height
                {
                    // Crop the image using the bounding box
                    let new_image = photon_rs::transform::crop(image, x, y, x2, y2);
                    self.try_output(ctx, CH_IMAGE, AgentData::new_image(new_image))
                        .context("Failed to output")?;
                }
            }
        }

        Ok(())
    }
}

// Image Scale Down
struct ImageScaleDownAgent {
    data: AsAgentData,
}

impl AsAgent for ImageScaleDownAgent {
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
        if ctx.ch() == CH_IMAGE {
            let config_width = self
                .config()
                .context("missing config")?
                .get_integer(CONFIG_MAX_WIDTH)
                .context("missing width")? as u32;
            let config_height = self
                .config()
                .context("missing config")?
                .get_integer(CONFIG_MAX_HEIGHT)
                .context("missing height")? as u32;

            let image: Option<&PhotonImage> = if data.is_image() {
                data.as_image()
            } else {
                data.get_image("image")
            };
            let Some(image) = image else {
                return Ok(());
            };

            let image_width = image.get_width();
            let image_height = image.get_height();

            if image_width <= config_width && image_height <= config_height {
                return Ok(());
            }

            let ratio_width = config_width as f32 / image_width as f32;
            let ratio_height = config_height as f32 / image_height as f32;
            let aspect_ratio = if ratio_width < ratio_height {
                ratio_width
            } else {
                ratio_height
            };

            let mut new_width = (image_width as f32 * aspect_ratio) as u32;
            if new_width == 0 {
                new_width = 1;
            }
            let mut new_height = (image_height as f32 * aspect_ratio) as u32;
            if new_height == 0 {
                new_height = 1;
            }

            // Scale down the image
            let new_image = photon_rs::transform::resize(
                &image,
                new_width,
                new_height,
                photon_rs::transform::SamplingFilter::Nearest,
            );

            self.try_output(ctx, CH_IMAGE, AgentData::new_image(new_image))
                .context("Failed to output")?;
        }

        Ok(())
    }
}

static CATEGORY: &str = "Core/Image";

static CH_BOUNDING: &str = "bounding";
static CH_IMAGE: &str = "image";

static CONFIG_MAX_WIDTH: &str = "width";
static CONFIG_MAX_HEIGHT: &str = "height";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    defs.insert(
        "$image_crop".to_string(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$image_crop",
            Some(new_boxed::<ImageCropAgent>),
        )
        .with_title("Image Crop")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_IMAGE, CH_BOUNDING])
        .with_outputs(vec![CH_IMAGE]),
    );

    defs.insert(
        "$image_scale_down".to_string(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$image_scale_down",
            Some(new_boxed::<ImageScaleDownAgent>),
        )
        .with_title("Image Scale Down")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_IMAGE])
        .with_outputs(vec![CH_IMAGE])
        .with_default_config(vec![
            (
                CONFIG_MAX_WIDTH.into(),
                AgentConfigEntry::new(AgentValue::new_integer(0), "integer"),
            ),
            (
                CONFIG_MAX_HEIGHT.into(),
                AgentConfigEntry::new(AgentValue::new_integer(0), "integer"),
            ),
        ]),
    );
}
