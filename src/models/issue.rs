use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueAnalysis {
    pub summary: String,
    pub requirements: Vec<String>,
    pub affected_areas: Vec<String>,
    pub possible_files: Vec<String>,
    pub implementation_approach: Vec<String>,
    pub testing_requirements: Vec<String>,
    pub complexity: Complexity,
    pub newcomer_suitability: String,
    pub uncertainties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    Low,
    Medium,
    High,
}
