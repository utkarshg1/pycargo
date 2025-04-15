use clap::Parser;
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

    /// Name of the GitHub repo (optional)
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
async fn main() {
    let args = Args::parse();

    // If the --version flag is used, exit early to avoid additional output
    if std::env::args().any(|arg| arg == "--version") {
        return;
    }

    validate_env_vars();

    // Check Git configuration at the start
    check_git_config("user.name", "name").await;
    check_git_config("user.email", "email").await;

    let dependencies = ["git", "pip"];
    for dep in dependencies {
        check_dependency(dep).await;
    }

    let project_name = &args.name;

    println!("📁 Creating project directory...");
    fs::create_dir(project_name)
        .await
        .expect("Failed to create project directory");
    env::set_current_dir(project_name).expect("Failed to change directory");

    println!("📝 Copying requirements.txt from template...");

    let template_content = match args.setup.as_str() {
        "basic" => BASIC_TEMPLATE,
        "advanced" => ADVANCED_TEMPLATE,
        "data-science" => DATASCIENCE_TEMPLATE,
        "blank" => "",
        _ => panic!("Invalid setup type. Use 'basic', 'advanced', 'data-science', or 'blank'."),
    };

    fs::write("requirements.txt", template_content)
        .await
        .expect("Failed to write requirements.txt from template");

    println!("🔧 Checking uv installation...");
    if run("uv", &["--version"]).await.is_err() {
        println!("uv is not installed. Installing uv...");
        run("pip", &["install", "uv"])
            .await
            .expect("Failed to install uv");
    }

    println!("🚀 Initializing uv... ");
    run("uv", &["init", "."])
        .await
        .expect("Failed to initialize uv");
    if tokio::fs::metadata(".gitignore").await.is_ok() {
        fs::remove_file(".gitignore").await.unwrap_or_else(|err| {
            panic!("❌ Failed to remove .gitignore: {}", err);
        });
    }

    println!("📦 Downloading .gitignore...");
    download_and_write_file(GITIGNORE_URL, ".gitignore").await;

    println!("📄 Downloading Apache LICENSE...");
    download_and_write_file(LICENSE_URL, "LICENSE").await;

    println!("🐍 Creating virtual environment...");
    run("uv", &["venv", ".venv"])
        .await
        .expect("Failed to create virtual environment");

    println!("📈 Upgrading pip...");
    run("uv", &["pip", "install", "--upgrade", "pip"])
        .await
        .expect("Failed to upgrade pip");

    println!("📥 Installing requirements...");
    run("uv", &["add", "-r", "requirements.txt"])
        .await
        .expect("Failed to add requirements");
    run("uv", &["sync"])
        .await
        .expect("Failed to sync dependencies");

    println!("🔧 Adding all files to Git...");
    git_command(&["add", "."])
        .await
        .expect("Failed to add files to Git");

    println!("🔧 Configuring Git line endings...");
    git_command(&["config", "core.autocrlf", "true"])
        .await
        .expect("Failed to configure Git line endings");

    println!("🔧 Commiting Git repo...");
    if let Err(err) = git_command(&["commit", "-m", "Initial commit"]).await {
        eprintln!("❌ Failed to commit changes: {}", err);
        return;
    }

    if let Some(repo_name) = args.github_repo {
        println!("☁️ Creating GitHub repo: {}", repo_name);
        create_github_repo(&repo_name, args.private).await;
        let username = get_git_username().await;
        git_command(&["branch", "-M", "main"])
            .await
            .expect("Failed to rename branch to main");
        git_command(&[
            "remote",
            "add",
            "origin",
            &format!("https://github.com/{}/{}.git", username, repo_name),
        ])
        .await
        .expect("Failed to add remote origin");
        git_command(&["push", "-u", "origin", "main"])
            .await
            .expect("Failed to push to remote repository");
    }

    println!("✅ Setup Completed 🐍");
}

fn validate_env_vars() {
    if env::var("GITHUB_TOKEN").is_err() {
        eprintln!(
            "❌ GITHUB_TOKEN environment variable is not set. Please set it before proceeding."
        );
        std::process::exit(1);
    }
}

async fn check_git_config(key: &str, prompt: &str) {
    let value = get_git_config(key).await;
    if value.is_empty() {
        println!(
            "Git global {} is not set. Please enter your {}:",
            key, prompt
        );
        let input = get_user_input();
        if let Err(err) = git_command(&["config", "--global", key, &input]).await {
            eprintln!("❌ Failed to set git {}: {}", key, err);
        }
    }
}

async fn run(cmd: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new(cmd).args(args).status().await?;
    if !status.success() {
        Err(format!("❌ Failed to run: {} {:?}", cmd, args).into())
    } else {
        Ok(())
    }
}

async fn git_command(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    run("git", args).await
}

async fn download_and_write_file(url: &str, filename: &str) {
    let body = reqwest::get(url).await.unwrap().text().await.unwrap();
    fs::write(filename, body).await.unwrap();
}

async fn create_github_repo(name: &str, private: bool) {
    let token = env::var("GITHUB_TOKEN").expect("Set GITHUB_TOKEN env var");
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.github.com/user/repos")
        .bearer_auth(token)
        .header("User-Agent", "pycargo")
        .json(&serde_json::json!({ "name": name, "private": private }))
        .send()
        .await;

    match res {
        Ok(response) if response.status().is_success() => {
            println!("✅ GitHub repository '{}' created successfully.", name);
        }
        Ok(response) => {
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            eprintln!("❌ Failed to create GitHub repo: {}", error_message);
        }
        Err(err) => {
            eprintln!("❌ Error while sending request to GitHub API: {}", err);
        }
    }
}

async fn get_git_username() -> String {
    let output = Command::new("git")
        .args(["config", "--get", "user.name"])
        .output()
        .await
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

async fn check_dependency(cmd: &str) {
    if run(cmd, &["--version"]).await.is_err() {
        eprintln!(
            "❌ Dependency '{}' is not installed. Please install it and try again.",
            cmd
        );
        std::process::exit(1);
    }
}

async fn get_git_config(key: &str) -> String {
    let output = Command::new("git")
        .args(["config", "--get", key])
        .output()
        .await
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn get_user_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
