use colorgrad::Gradient;

pub struct GlobalResource {
    pub gradient: colorgrad::CatmullRomGradient,
}

/* impl Default for GlobalResource {
    fn default() -> Self {
        GlobalResource {
            gradient: colorgrad::ma(),
        }
    }
}

#[server]
pub async fn try_new() -> Result<GlobalResource, ServerFnError> {
    let g = colorgrad::GradientBuilder::new()
        .html_colors(&["deeppink", "gold", "seagreen"])
        .build::<colorgrad::CatmullRomGradient>();
    match g {
        Ok(gm) => Ok(GlobalResource { gradient: g }),
        Err(_) => Err(ServerFnError::Request(
            "Failed to create GlobalResource".to_string(),
        )),
    }
} */