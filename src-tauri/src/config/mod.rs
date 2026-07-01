use crate::utils::{VoltError, VoltResult};
use std::collections::HashMap;
use std::path::Path;

pub struct ConfigGenerator;

impl ConfigGenerator {
    pub fn generate(
        template_name: &str,
        vars: &HashMap<String, String>,
        templates_override_dir: Option<&Path>,
    ) -> VoltResult<String> {
        let template_content = Self::load_template(template_name, templates_override_dir)?;

        let mut rendered = template_content;
        for (key, value) in vars {
            rendered = rendered.replace(&format!("{{{{ {} }}}}", key), value);
        }

        Ok(rendered)
    }

    pub fn generate_to_file(
        template_name: &str,
        vars: &HashMap<String, String>,
        templates_override_dir: Option<&Path>,
        output_path: &Path,
    ) -> VoltResult<()> {
        let content = Self::generate(template_name, vars, templates_override_dir)?;
        std::fs::write(output_path, &content).map_err(VoltError::Io)
    }

    fn load_template(name: &str, override_dir: Option<&Path>) -> VoltResult<String> {
        if let Some(dir) = override_dir {
            let path = dir.join(name);
            if path.exists() {
                return std::fs::read_to_string(&path).map_err(VoltError::Io);
            }
        }

        match name {
            "nginx.conf.tpl" => Ok(include_str!("templates/nginx.conf.tpl").to_string()),
            "php.ini.tpl" => Ok(include_str!("templates/php.ini.tpl").to_string()),
            "my.cnf.tpl" => Ok(include_str!("templates/my.cnf.tpl").to_string()),
            _ => Err(VoltError::Config(format!("Unknown template: {}", name))),
        }
    }
}
