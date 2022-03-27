
pub mod template_engine {
    use std::collections::HashMap;

    #[derive(Clone)]
    pub struct TemplateEngine {}

    impl TemplateEngine {
        pub fn render(&self, template: &str, context: HashMap<String, String>) -> String {
            let mut template_res = template.to_string();
            for (key, val) in context {
                let key_ph = format!("${{{}}}", key);
                template_res = template_res.replace(key_ph.as_str(), val.as_str())
            }
            template_res
        }
    }
}
