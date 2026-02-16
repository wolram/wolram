use crate::state_machine::ModelTier;

/// Routes a job description to an appropriate skill/agent type.
pub struct SkillRouter;

impl SkillRouter {
    /// Keyword-based skill assignment from a job description.
    pub fn route(description: &str) -> String {
        let lower = description.to_lowercase();
        if lower.contains("test") {
            "testing".to_string()
        } else if lower.contains("refactor") {
            "refactoring".to_string()
        } else if lower.contains("doc") {
            "documentation".to_string()
        } else if lower.contains("fix") || lower.contains("bug") {
            "bug_fix".to_string()
        } else {
            "code_generation".to_string()
        }
    }
}

/// Selects a model tier based on task complexity inferred from the description.
pub struct ModelSelector;

impl ModelSelector {
    /// Complexity-based model selection from a job description.
    pub fn select(description: &str) -> ModelTier {
        let lower = description.to_lowercase();
        let simple_keywords = ["rename", "format", "typo"];
        let complex_keywords = ["architect", "refactor", "redesign"];

        if simple_keywords.iter().any(|k| lower.contains(k)) || lower.len() < 20 {
            ModelTier::Haiku
        } else if complex_keywords.iter().any(|k| lower.contains(k)) {
            ModelTier::Opus
        } else {
            ModelTier::Sonnet
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_testing() {
        assert_eq!(SkillRouter::route("Write unit tests for the parser"), "testing");
    }

    #[test]
    fn route_refactoring() {
        assert_eq!(SkillRouter::route("Refactor the auth module"), "refactoring");
    }

    #[test]
    fn route_documentation() {
        assert_eq!(SkillRouter::route("Add docs for the API"), "documentation");
    }

    #[test]
    fn route_bug_fix() {
        assert_eq!(SkillRouter::route("Fix the login bug"), "bug_fix");
        assert_eq!(SkillRouter::route("Debug the crash"), "bug_fix");
    }

    #[test]
    fn route_default() {
        assert_eq!(SkillRouter::route("Implement hero section layout"), "code_generation");
    }

    #[test]
    fn select_haiku_for_simple() {
        assert_eq!(ModelSelector::select("rename variable"), ModelTier::Haiku);
        assert_eq!(ModelSelector::select("fix typo in readme"), ModelTier::Haiku);
        assert_eq!(ModelSelector::select("format code"), ModelTier::Haiku);
    }

    #[test]
    fn select_haiku_for_short() {
        assert_eq!(ModelSelector::select("add a button"), ModelTier::Haiku);
    }

    #[test]
    fn select_opus_for_complex() {
        assert_eq!(ModelSelector::select("architect the new payment system"), ModelTier::Opus);
        assert_eq!(ModelSelector::select("refactor the entire auth module"), ModelTier::Opus);
        assert_eq!(ModelSelector::select("redesign the database schema for scaling"), ModelTier::Opus);
    }

    #[test]
    fn select_sonnet_default() {
        assert_eq!(ModelSelector::select("implement the user profile page"), ModelTier::Sonnet);
    }
}
