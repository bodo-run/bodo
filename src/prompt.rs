use dialoguer::{Confirm, Input, Select};

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

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_manager_creation() {
        let prompt_manager = PromptManager::new();
        // Just verify it can be created
        assert!(true);
    }

    // Note: We can't easily test the interactive methods (confirm, input, select)
    // in automated tests because they require user interaction.
    // In a real-world scenario, we might want to:
    // 1. Mock the dialoguer crate
    // 2. Create a trait for the prompt interface and use a test double
    // 3. Add integration tests for the interactive features
}
