use anyhow::{Context, Result};
use clap::Parser;
use colored::*; // Add this at the top for colored output
use std::env;
use std::io;
use tokio::fs;
use tokio::process::Command;

const GITIGNORE_URL: &str =
    "https://raw.githubusercontent.com/github/gitignore/main/Python.gitignore";
const LICENSE_URL: &str = "https://www.apache.org/licenses/LICENSE-2.0.txt";

// Embed the template files into the binary using `include_str!`
const BASIC_TEMPLATE: &str = include_str!("../templates/basic.txt");
const ADVANCED_TEMPLATE: &str = include_str!("../templates/advanced.txt");
const DATASCIENCE_TEMPLATE: &str = include_str!("../templates/datascience.txt");

/// PyCargo – Bootstrap a Python Data Science Project
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Name of the project directory
    name: String,

    /// Name of the GitHub repo (optional, inferred from name if not provided)
    #[arg(long)]
    github_repo: Option<String>,

    /// Setup type: basic, advanced, data-science, or blank
    #[arg(long, default_value = "advanced")]
    setup: String,

    /// Specify if the GitHub repository should be private
    #[arg(long)]
    private: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Check directory existence
    let project_name = &args.name;
    if fs::metadata(project_name).await.is_ok() {
        anyhow::bail!("{} Directory '{}' already exists", "❌".red(), project_name);
    }

    // Check Git configuration
    check_git_config("user.name", "name").await?;
    check_git_config("user.email", "email").await?;

    // Check dependencies
    check_uv_installation().await?;

    // Add header for project setup
    println!("\n=== {} ===", "Project Setup".bold().underline());

    // Create project structure
    println!("{} Creating project directory...", "📁".blue());
    fs::create_dir(project_name).await?;
    env::set_current_dir(project_name)?;

    // Setup environment
    setup_environment().await?;

    // Setup requirements.txt
    println!(
        "{} Creating requirements.txt from template...",
        "📝".green()
    );
    create_requirements_file(&args.setup).await?;

    // Download additional files
    println!("{} Downloading .gitignore...", "📦".yellow());
    download_and_write_file(GITIGNORE_URL, ".gitignore").await?;

    println!("{} Downloading Apache LICENSE...", "📄".cyan());
    download_and_write_file(LICENSE_URL, "LICENSE").await?;

    // Initialize Git
    println!("{} Initializing Git repository...", "🔧".blue());
    initialize_git_repo().await?;

    // Handle GitHub integration
    if let Some(repo_name) = args.github_repo {
        // Add header for GitHub integration
        println!("\n=== {} ===", "GitHub Integration".bold().underline());

        validate_env_vars()?;
        create_github_repo(&repo_name, args.private).await?;
        setup_github_remote(&repo_name).await?;

        // Extract GitHub username from git global config
        let output = Command::new("git")
            .args(["config", "--global", "user.name"])
            .output()
            .await
            .context("Failed to retrieve GitHub username from git config")?;

        let github_username = String::from_utf8(output.stdout)
            .context("Failed to parse GitHub username from git config output")?
            .trim()
            .to_string();

        let remote_url = format!("https://github.com/{}/{}.git", github_username, repo_name);

        // Add summary at the end
        println!("\n{} Setup Completed 🐍", "✅".green());
        println!(
            "\nTo activate the virtual environment, run: {}",
            ".venv\\Scripts\\activate".bold()
        );
        println!("\nFinal repository link: {}", remote_url);
    } else {
        // Add summary at the end
        println!("\n{} Setup Completed 🐍", "✅".green());
        println!(
            "\nTo activate the virtual environment, run: {}",
            ".venv\\Scripts\\activate".bold()
        );
    }

    Ok(())
}

async fn check_uv_installation() -> Result<()> {
    println!("🔧 Checking uv installation...");
    if run("uv", &["--version"]).await.is_err() {
        println!("uv is not installed. Installing uv...");
        run("pip", &["install", "uv"]).await?;
    }
    Ok(())
}

async fn setup_environment() -> Result<()> {
    println!("🚀 Initializing uv...");
    run("uv", &["init", "."]).await?;

    println!("🐍 Creating virtual environment...");
    run("uv", &["venv", ".venv"]).await
}

async fn create_requirements_file(setup_type: &str) -> Result<()> {
    let content = match setup_type {
        "basic" => BASIC_TEMPLATE,
        "advanced" => ADVANCED_TEMPLATE,
        "data-science" => DATASCIENCE_TEMPLATE,
        "blank" => "",
        _ => {
            anyhow::bail!("Invalid setup type. Use 'basic', 'advanced', 'data-science', or 'blank'")
        }
    };

    fs::write("requirements.txt", content).await?;

    if setup_type != "blank" {
        println!("📥 Installing requirements...");
        run("uv", &["add", "-r", "requirements.txt"]).await?;
        run("uv", &["sync"]).await?;
    }

    Ok(())
}

async fn initialize_git_repo() -> Result<()> {
    println!("🔧 Initializing Git repository...");
    git_command(&["init"]).await?;
    git_command(&["config", "core.autocrlf", "true"]).await?;
    git_command(&["add", "."]).await?;

    println!("{} Committing initial state...", "🔧".blue());
    git_command(&["commit", "-m", "Initial commit"]).await?;

    Ok(())
}

async fn setup_github_remote(repo_name: &str) -> Result<()> {
    git_command(&["branch", "-M", "main"]).await?;

    // Extract GitHub username from git global config
    let output = Command::new("git")
        .args(["config", "--global", "user.name"])
        .output()
        .await
        .context("Failed to retrieve GitHub username from git config")?;

    let github_username = String::from_utf8(output.stdout)
        .context("Failed to parse GitHub username from git config output")?
        .trim()
        .to_string();

    let remote_url = format!("https://github.com/{}/{}.git", github_username, repo_name);

    println!("🔗 Adding GitHub remote...");
    git_command(&["remote", "add", "origin", &remote_url]).await?;
    git_command(&["push", "-u", "origin", "main"]).await?;

    Ok(())
}

async fn download_and_write_file(url: &str, filename: &str) -> Result<()> {
    let response = reqwest::get(url).await.context("Failed to download file")?;
    if !response.status().is_success() {
        anyhow::bail!("HTTP error: {}", response.status());
    }
    let body = response
        .text()
        .await
        .context("Failed to read response body")?;
    fs::write(filename, body)
        .await
        .context("Failed to write file")?;
    Ok(())
}

async fn check_git_config(key: &str, prompt: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["config", "--get", key])
        .output()
        .await?;

    if output.stdout.is_empty() {
        println!(
            "Git {} is not configured. Please enter your {}:",
            key, prompt
        );
        let input = get_user_input();
        git_command(&["config", "--global", key, &input]).await?;
    }
    Ok(())
}

async fn create_github_repo(name: &str, private: bool) -> Result<()> {
    let token = env::var("GITHUB_TOKEN").context("GITHUB_TOKEN not set")?;
    let client = reqwest::Client::new();

    let response = client
        .post("https://api.github.com/user/repos")
        .bearer_auth(token)
        .header("User-Agent", "pycargo")
        .json(&serde_json::json!({ "name": name, "private": private }))
        .send()
        .await
        .context("Failed to create GitHub repository")?;

    if !response.status().is_success() {
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!("GitHub API error: {}", error_body);
    }

    println!("✅ Created GitHub repository: {}", name);
    Ok(())
}

async fn run(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .await
        .with_context(|| format!("Failed to execute: {} {}", cmd, args.join(" ")))?;

    if !status.success() {
        anyhow::bail!("Command failed: {} {}", cmd, args.join(" "));
    }
    Ok(())
}

async fn git_command(args: &[&str]) -> Result<()> {
    run("git", args).await
}

fn validate_env_vars() -> Result<()> {
    if env::var("GITHUB_TOKEN").is_err() {
        anyhow::bail!("GITHUB_TOKEN environment variable is not set");
    }
    Ok(())
}

fn get_user_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}
