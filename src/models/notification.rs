use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub repository: String,
    pub issue_title: String,
    pub issue_url: String,
    pub issue_labels: Vec<String>,
    pub summary: String,
    pub technical_interpretation: String,
    pub possible_files: Vec<String>,
    pub implementation_approach: Vec<String>,
    pub complexity: String,
    pub uncertainties: Vec<String>,
}
