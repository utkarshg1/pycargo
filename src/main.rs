use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::io;
use std::time::Duration;
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
    #[arg(short, long)]
    name: String,

    /// Flag to indicate if a GitHub repo should be created
    #[arg(short = 'g', long)]
    github_repo: bool,

    /// Optional custom name for the GitHub repo
    #[arg(long, value_name = "GITHUB_REPO_NAME")]
    github_repo_name: Option<String>,

    /// Setup type: basic, advanced, data-science, or blank
    #[arg(short = 's', long, default_value = "advanced")]
    setup: String,

    /// Specify if the GitHub repository should be private
    #[arg(short = 'p', long)]
    private: bool,
}

fn spinner_style() -> ProgressStyle {
    ProgressStyle::default_spinner()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        .template("{spinner} {msg}")
        .expect("Failed to set spinner template")
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Early exit for version flag
    if std::env::args().any(|arg| arg == "--version") {
        return Ok(());
    }

    println!("{}", "=== 📁 Project Setup ===".bold().blue());

    let project_name = &args.name;
    if fs::metadata(project_name).await.is_ok() {
        anyhow::bail!(
            "{}",
            format!("❌ Directory '{}' already exists", project_name).red()
        );
    }

    // Create project directory
    fs::create_dir(project_name).await?;
    println!(
        "{}",
        format!("✅ Created project directory: {}", project_name).green()
    );

    // Check Git configuration
    check_git_config("user.name", "name").await?;
    check_git_config("user.email", "email").await?;

    // Check dependencies
    check_uv_installation().await?;

    println!("\n{}", "=== 🚀 Environment Setup ===".bold().blue());

    // Change into project directory
    env::set_current_dir(project_name)?;

    // Setup environment
    setup_environment().await?;

    println!("{}", "✅ Initialized project with uv".green());
    println!("{}", "✅ Created virtual environment".green());
    println!("{}", "Activate with: .venv\\Scripts\\activate".yellow());

    // Setup requirements.txt
    create_requirements_file(&args.setup).await?;
    println!("{}", "✅ Created requirements.txt from template".green());
    println!("{}", "✅ Installed requirements".green());

    println!("\n{}", "=== 📦 File Downloads ===".bold().blue());

    // Download additional files
    download_and_write_file(GITIGNORE_URL, ".gitignore").await?;
    println!("{}", "✅ Downloaded .gitignore".green());

    download_and_write_file(LICENSE_URL, "LICENSE").await?;
    println!("{}", "✅ Downloaded Apache LICENSE".green());

    println!("\n{}", "=== 🔧 Git Setup ===".bold().blue());

    // Initialize Git
    initialize_git_repo().await?;
    println!("{}", "✅ Initialized Git repository".green());
    println!("{}", "✅ Committed initial state".green());
    println!(
        "{}",
        "Files: .gitignore, LICENSE, README.md, main.py, etc.".yellow()
    );

    // Handle GitHub integration
    if args.github_repo {
        let repo_name = args
            .github_repo_name
            .clone()
            .unwrap_or_else(|| project_name.clone());

        validate_env_vars()?;
        create_github_repo(&repo_name, args.private).await?;
        let remote_url = setup_github_remote(&repo_name).await?;
        println!(
            "{}",
            format!(
                "✅ GitHub repository created: {}",
                remote_url.trim_end_matches(".git")
            )
            .green()
        );
    }

    println!("\n{}", "✅ Setup Completed 🐍".bold().green());

    println!(
        "\n{}",
        "To activate the virtual environment, run:".bold().blue()
    );
    println!("{}", ".venv\\Scripts\\activate".yellow());

    Ok(())
}

async fn check_uv_installation() -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style());
    spinner.set_message("Checking uv installation...");
    spinner.enable_steady_tick(Duration::from_millis(100));

    if run("uv", &["--version"]).await.is_err() {
        spinner.finish_and_clear();
        println!("uv not found. Installing uv...");
        run("pip", &["install", "uv"]).await?;
        println!("{}", "✅ uv installed".green());
    } else {
        spinner.finish_and_clear();
        println!("{}", "✅ uv is already installed".green());
    }
    Ok(())
}

async fn setup_environment() -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style());
    spinner.set_message("Initializing uv...");
    spinner.enable_steady_tick(Duration::from_millis(100));
    run("uv", &["init", "."]).await?;
    spinner.finish_and_clear();
    println!("{}", "✅ uv initialized".green());

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style());
    spinner.set_message("Creating virtual environment...");
    spinner.enable_steady_tick(Duration::from_millis(100));
    run("uv", &["venv", ".venv"]).await?;
    spinner.finish_and_clear();
    println!("{}", "✅ virtual environment created".green());

    Ok(())
}

async fn create_requirements_file(setup_type: &str) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style());
    spinner.set_message("Writing requirements.txt...");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let content = match setup_type {
        "basic" => BASIC_TEMPLATE,
        "advanced" => ADVANCED_TEMPLATE,
        "data-science" => DATASCIENCE_TEMPLATE,
        "blank" => "",
        _ => {
            spinner.finish_and_clear();
            anyhow::bail!("Invalid setup type. Use 'basic', 'advanced', 'data-science', or 'blank'")
        }
    };

    fs::write("requirements.txt", content).await?;
    spinner.finish_and_clear();
    println!("{}", "✅ requirements.txt created".green());

    if setup_type != "blank" {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(spinner_style());
        spinner.set_message("Installing requirements...");
        spinner.enable_steady_tick(Duration::from_millis(100));
        run("uv", &["add", "-r", "requirements.txt"]).await?;
        run("uv", &["sync"]).await?;
        spinner.finish_and_clear();
        println!("{}", "✅ requirements installed".green());
    }

    Ok(())
}

async fn download_and_write_file(url: &str, filename: &str) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style());
    spinner.set_message(format!("Downloading {}...", filename));
    spinner.enable_steady_tick(Duration::from_millis(100));

    let response = reqwest::get(url).await.context("Failed to download file")?;
    if !response.status().is_success() {
        spinner.finish_and_clear();
        anyhow::bail!("HTTP error: {}", response.status());
    }
    let body = response
        .text()
        .await
        .context("Failed to read response body")?;
    fs::write(filename, body)
        .await
        .context("Failed to write file")?;
    spinner.finish_and_clear();
    println!("{}", format!("✅ Downloaded {}", filename).green());
    Ok(())
}

async fn initialize_git_repo() -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style());
    spinner.set_message("Initializing Git repository...");
    spinner.enable_steady_tick(Duration::from_millis(100));

    git_command(&["init"]).await?;
    git_command(&["config", "core.autocrlf", "true"]).await?;
    git_command(&["add", "."]).await?;
    git_command(&["commit", "-m", "Initial commit"]).await?;

    spinner.finish_and_clear();
    println!("{}", "✅ Git repository initialized and committed".green());
    Ok(())
}

async fn setup_github_remote(repo_name: &str) -> Result<String> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style());
    spinner.set_message("Setting up GitHub remote...");
    spinner.enable_steady_tick(Duration::from_millis(100));

    git_command(&["branch", "-M", "main"]).await?;
    let username = get_git_username().await?;
    let remote_url = format!("https://github.com/{}/{}.git", username, repo_name);
    git_command(&["remote", "add", "origin", &remote_url]).await?;
    git_command(&["push", "-u", "origin", "main"]).await?;

    spinner.finish_and_clear();
    println!("{}", "✅ GitHub remote configured".green());
    Ok(remote_url)
}

async fn create_github_repo(name: &str, private: bool) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style());
    spinner.set_message("Creating GitHub repository via API...");
    spinner.enable_steady_tick(Duration::from_millis(100));

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
        spinner.finish_and_clear();
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!("GitHub API error: {}", error_body);
    }

    spinner.finish_and_clear();
    println!(
        "{}",
        format!("✅ GitHub repository '{}' created", name).green()
    );
    Ok(())
}

async fn run(cmd: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .await
        .with_context(|| format!("Failed to execute: {} {}", cmd, args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "{}",
            format!(
                "Command failed: {} {}\nError Output: {}",
                cmd,
                args.join(" "),
                stderr
            )
            .red()
        );
    }

    Ok(())
}

async fn git_command(args: &[&str]) -> Result<()> {
    run("git", args).await
}

fn validate_env_vars() -> Result<()> {
    if env::var("GITHUB_TOKEN").is_err() {
        anyhow::bail!("{}", "GITHUB_TOKEN environment variable is not set".red());
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

/// Retrieves the GitHub username from global git config
async fn get_git_username() -> Result<String> {
    let output = Command::new("git")
        .args(["config", "--global", "user.name"])
        .output()
        .await
        .context("Failed to retrieve GitHub username from git config")?;

    let username = String::from_utf8(output.stdout)
        .context("Failed to parse GitHub username from git config output")?
        .trim()
        .to_string();

    Ok(username)
}

/// Checks and sets git global configuration if missing, with spinner feedback
async fn check_git_config(key: &str, prompt: &str) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style());
    spinner.set_message(format!("Checking git config for {}...", key));
    spinner.enable_steady_tick(Duration::from_millis(100));

    let output = Command::new("git")
        .args(["config", "--get", key])
        .output()
        .await
        .context("Failed to get git config")?;

    if output.stdout.is_empty() {
        spinner.finish_and_clear();
        println!(
            "Git {} is not configured. Please enter your {}:",
            key, prompt
        );
        let input = get_user_input();

        let spinner2 = ProgressBar::new_spinner();
        spinner2.set_style(spinner_style());
        spinner2.set_message(format!("Setting git {}...", key));
        spinner2.enable_steady_tick(Duration::from_millis(100));
        git_command(&["config", "--global", key, &input]).await?;
        spinner2.finish_and_clear();
        println!("{}", format!("✅ Git {} configured", key).green());
    } else {
        spinner.finish_and_clear();
        println!("{}", format!("✅ Git {} already configured", key).green());
    }

    Ok(())
}
