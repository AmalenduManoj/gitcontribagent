use std::process::Command;
use std::time::Duration;

/// Check if Ollama is installed on the system.
pub fn is_ollama_installed() -> bool {
    Command::new("ollama")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Ollama server is responding.
pub async fn is_ollama_running(base_url: &str) -> bool {
    let client = reqwest::Client::new();
    client
        .get(format!("{}/api/tags", base_url))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Start Ollama server in the background.
pub fn start_ollama_server() -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::process::Stdio;
        Command::new("ollama")
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    }
    Ok(())
}

/// Check if a specific model is available in Ollama.
pub async fn is_model_available(base_url: &str, model: &str) -> bool {
    let client = reqwest::Client::new();
    match client.get(format!("{}/api/tags", base_url)).send().await {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                    return models.iter().any(|m| {
                        m.get("name")
                            .and_then(|n| n.as_str())
                            .map(|n| n == model || n.starts_with(&format!("{}:", model)))
                            .unwrap_or(false)
                    });
                }
            }
            false
        }
        Err(_) => false,
    }
}

/// Pull a model from Ollama with progress indication.
pub async fn pull_model(base_url: &str, model: &str) -> anyhow::Result<()> {
    println!("Pulling model {}... (this may take a while)", model);
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/pull", base_url))
        .json(&serde_json::json!({ "name": model, "stream": false }))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to pull model {}: {}", model, resp.status());
    }

    println!("Model {} pulled successfully.", model);
    Ok(())
}

/// Ensure Ollama is running and the required model is available.
/// Returns Ok(()) if everything is ready.
pub async fn ensure_ollama_ready(base_url: &str, model: &str) -> anyhow::Result<()> {
    if !is_ollama_installed() {
        println!("Ollama is not installed.");
        println!("Install it with: curl -fsSL https://ollama.com/install.sh | sh");
        anyhow::bail!("Ollama not installed");
    }

    if !is_ollama_running(base_url).await {
        println!("Starting Ollama server...");
        start_ollama_server()?;
        // Wait for server to be ready
        for _ in 0..30 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if is_ollama_running(base_url).await {
                break;
            }
        }
        if !is_ollama_running(base_url).await {
            anyhow::bail!("Ollama server failed to start");
        }
    }

    if !is_model_available(base_url, model).await {
        pull_model(base_url, model).await?;
    }

    println!("Ollama is ready with model: {}", model);
    Ok(())
}
