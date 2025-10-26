use crate::color::ColorPalette;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use tera::{Tera, Context as TeraContext};
use tracing::info;

/// Template renderer for generating theme files
pub struct TemplateRenderer {
    tera: Tera,
}

impl TemplateRenderer {
    /// Create a new template renderer
    /// If template_dir is provided, loads templates from that directory
    /// Otherwise, uses embedded templates
    pub fn new(template_dir: Option<&Path>) -> Result<Self> {
        let mut tera = if let Some(dir) = template_dir {
            let pattern = dir.join("**/*.{ini,css,json,toml}");
            Tera::new(pattern.to_str().unwrap())
                .with_context(|| format!("Failed to load templates from {:?}", dir))?
        } else {
            // Use embedded templates
            let mut tera = Tera::default();

            // Load embedded templates
            tera.add_raw_template(
                "omarcord.theme.css",
                include_str!("../templates/omarcord.theme.css"),
            )?;
            tera.add_raw_template(
                "omarchify-colors.ini",
                include_str!("../templates/omarchify-colors.ini"),
            )?;

            tera
        };

        // Disable autoescape for all templates
        tera.autoescape_on(vec![]);

        Ok(TemplateRenderer { tera })
    }

    /// Render a template with the given color palette
    pub fn render(
        &self,
        template_name: &str,
        palette: &ColorPalette,
        extra_vars: &HashMap<String, String>,
    ) -> Result<String> {
        let mut context = TeraContext::new();

        // Add color values to context (with # for CSS)
        // Also add _hex versions without # for INI files
        macro_rules! add_color {
            ($field:ident) => {
                if let Some(color) = &palette.$field {
                    context.insert(stringify!($field), color.hex());
                    context.insert(&format!("{}_hex", stringify!($field)), color.hex_no_hash());
                }
            };
        }

        add_color!(background);
        add_color!(foreground);
        add_color!(black);
        add_color!(red);
        add_color!(green);
        add_color!(yellow);
        add_color!(blue);
        add_color!(magenta);
        add_color!(cyan);
        add_color!(white);
        add_color!(bright_black);
        add_color!(bright_red);
        add_color!(bright_green);
        add_color!(bright_yellow);
        add_color!(bright_blue);
        add_color!(bright_magenta);
        add_color!(bright_cyan);
        add_color!(bright_white);
        add_color!(cursor);
        add_color!(selection_background);
        add_color!(selection_foreground);

        // Add custom colors
        for (key, color) in &palette.custom {
            context.insert(key, color.hex());
        }

        // Add extra variables
        for (key, value) in extra_vars {
            context.insert(key, value);
        }

        // Determine the full template name (add extension if needed)
        let full_template_name = if template_name.contains('.') {
            template_name.to_string()
        } else {
            // Try common extensions in priority order
            // .theme.css comes before -colors.css for themes like omarcord
            if self.tera.get_template_names().any(|n| n == format!("{}.theme.css", template_name)) {
                format!("{}.theme.css", template_name)
            } else if self.tera.get_template_names().any(|n| n == format!("{}.ini", template_name)) {
                format!("{}.ini", template_name)
            } else if self.tera.get_template_names().any(|n| n == format!("{}-colors.ini", template_name)) {
                format!("{}-colors.ini", template_name)
            } else if self.tera.get_template_names().any(|n| n == format!("{}-colors.css", template_name)) {
                format!("{}-colors.css", template_name)
            } else if self.tera.get_template_names().any(|n| n == format!("{}.css", template_name)) {
                format!("{}.css", template_name)
            } else {
                template_name.to_string()
            }
        };

        info!("Rendering template: {}", full_template_name);

        self.tera
            .render(&full_template_name, &context)
            .with_context(|| format!("Failed to render template: {}", full_template_name))
    }

    /// Get list of available template names
    #[allow(dead_code)]
    pub fn available_templates(&self) -> Vec<String> {
        self.tera.get_template_names().map(|s| s.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    #[test]
    fn test_template_renderer() {
        let renderer = TemplateRenderer::new(None).unwrap();
        let templates = renderer.available_templates();

        assert!(templates.contains(&"omarcord.theme.css".to_string()));
        assert!(templates.contains(&"omarchify-colors.ini".to_string()));
    }

    #[test]
    fn test_render_omarchify() {
        let renderer = TemplateRenderer::new(None).unwrap();
        let mut palette = ColorPalette::default();
        palette.background = Some(Color::new("#eff1f5").unwrap());
        palette.foreground = Some(Color::new("#4c4f69").unwrap());
        palette.bright_green = Some(Color::new("#40a02b").unwrap());
        palette.green = Some(Color::new("#40a02b").unwrap());

        let result = renderer.render("omarchify-colors", &palette, &HashMap::new()).unwrap();

        assert!(result.contains("[Omarchify]"));
        assert!(result.contains("accent"));
        assert!(result.contains("40a02b")); // hex without #
    }

    #[test]
    fn test_render_omarcord() {
        let renderer = TemplateRenderer::new(None).unwrap();
        let mut palette = ColorPalette::default();
        palette.background = Some(Color::new("#1a1b26").unwrap());
        palette.foreground = Some(Color::new("#c0caf5").unwrap());
        palette.blue = Some(Color::new("#7aa2f7").unwrap());
        palette.red = Some(Color::new("#f7768e").unwrap());
        palette.green = Some(Color::new("#9ece6a").unwrap());

        let result = renderer.render("omarcord", &palette, &HashMap::new()).unwrap();

        assert!(result.contains("@import"));
        assert!(result.contains("system24"));
        assert!(result.contains("--text-"));
        assert!(result.contains("--bg-"));
        assert!(result.contains("#1a1b26"));
        assert!(result.contains("#c0caf5"));
    }
}
