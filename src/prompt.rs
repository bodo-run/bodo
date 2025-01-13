use dialoguer::{Input, Select, Confirm};

pub struct PromptManager;

impl PromptManager {
    pub fn new() -> Self {
        Self
    }

    pub fn confirm(&self, message: &str) -> bool {
        Confirm::new()
            .with_prompt(message)
            .default(true)
            .interact()
            .unwrap_or(false)
    }

    pub fn input(&self, message: &str) -> Option<String> {
        Input::<String>::new()
            .with_prompt(message)
            .interact_text()
            .ok()
    }

    pub fn select(&self, message: &str, options: &[String]) -> Option<usize> {
        Select::new()
            .with_prompt(message)
            .items(options)
            .default(0)
            .interact()
            .ok()
    }
} 