use github_contribution_agent::models::issue::{Complexity, IssueAnalysis};
use github_contribution_agent::models::notification::Notification;

#[test]
fn issue_analysis_serializes_to_json() {
    let analysis = IssueAnalysis {
        summary: "Fix null pointer in parser".into(),
        requirements: vec!["Handle empty input".into()],
        affected_areas: vec!["parser".into()],
        possible_files: vec!["src/parser.rs".into()],
        implementation_approach: vec!["Add null check".into()],
        testing_requirements: vec!["Unit test for empty input".into()],
        complexity: Complexity::Low,
        newcomer_suitability: "Suitable for newcomers".into(),
        uncertainties: vec![],
    };

    let json = serde_json::to_value(&analysis).unwrap();
    assert_eq!(json["summary"], "Fix null pointer in parser");
    assert_eq!(json["complexity"], "low");
    assert_eq!(json["requirements"][0], "Handle empty input");
}

#[test]
fn complexity_serializes_lowercase() {
    assert_eq!(
        serde_json::to_value(Complexity::Low).unwrap(),
        serde_json::json!("low")
    );
    assert_eq!(
        serde_json::to_value(Complexity::Medium).unwrap(),
        serde_json::json!("medium")
    );
    assert_eq!(
        serde_json::to_value(Complexity::High).unwrap(),
        serde_json::json!("high")
    );
}

#[test]
fn complexity_roundtrips() {
    for c in [Complexity::Low, Complexity::Medium, Complexity::High] {
        let json = serde_json::to_value(&c).unwrap();
        let back: Complexity = serde_json::from_value(json).unwrap();
        assert_eq!(c, back);
    }
}

#[test]
fn notification_struct_roundtrips() {
    let notification = Notification {
        repository: "owner/repo".into(),
        issue_title: "Bug in parser".into(),
        issue_url: "https://github.com/owner/repo/issues/1".into(),
        issue_labels: vec!["bug".into(), "good first issue".into()],
        summary: "Parser crashes on empty input".into(),
        technical_interpretation: "Null dereference in parse()".into(),
        possible_files: vec!["src/parser.rs".into()],
        implementation_approach: vec!["Add null check".into()],
        complexity: "low".into(),
        uncertainties: vec!["Need to confirm with maintainer".into()],
    };

    let json = serde_json::to_value(&notification).unwrap();
    let back: Notification = serde_json::from_value(json).unwrap();

    assert_eq!(back.repository, "owner/repo");
    assert_eq!(back.issue_title, "Bug in parser");
    assert_eq!(back.complexity, "low");
    assert_eq!(back.issue_labels.len(), 2);
}
