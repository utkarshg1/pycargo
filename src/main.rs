use clap::Parser;
use std::env;
use tokio::fs;
use tokio::process::Command;

/// PyCargo – Bootstrap a Python Data Science Project
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Name of the project directory
    name: String,

    /// Name of the GitHub repo (optional)
    #[arg(long)]
    github_repo: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let project_name = &args.name;

    println!("📁 Creating project directory...");
    fs::create_dir(project_name)
        .await
        .expect("Failed to create project directory");
    env::set_current_dir(project_name).expect("Failed to change directory");

    println!("📝 Writing requirements.txt...");
    let requirements = r#"numpy
pandas
matplotlib
seaborn
ipykernel
plotly
nbformat
requests
beautifulsoup4
pydantic
"#;
    fs::write("requirements.txt", requirements)
        .await
        .expect("Failed to write requirements.txt");

    println!("🚀 Initializing uv... ");
    run("pip", &["install", "uv"])
        .await
        .expect("Failed to install uv");
    run("uv", &["init", "."])
        .await
        .expect("Failed to initialize uv");
    if tokio::fs::metadata(".gitignore").await.is_ok() {
        fs::remove_file(".gitignore").await.unwrap_or_else(|err| {
            panic!("❌ Failed to remove .gitignore: {}", err);
        });
    }

    println!("📦 Downloading .gitignore...");
    download_file(
        "https://raw.githubusercontent.com/github/gitignore/main/Python.gitignore",
        ".gitignore",
    )
    .await;

    println!("📄 Downloading Apache LICENSE...");
    download_file("https://www.apache.org/licenses/LICENSE-2.0.txt", "LICENSE").await;

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
    run("git", &["add", "."])
        .await
        .expect("Failed to add files to Git");

    println!("🔧 Configuring Git line endings...");
    run("git", &["config", "core.autocrlf", "true"])
        .await
        .expect("Failed to configure Git line endings");

    println!("🔧 Commiting Git repo...");
    if let Err(err) = run("git", &["commit", "-m", "Initial commit"]).await {
        eprintln!("❌ Failed to commit changes: {}", err);
        return;
    }

    if let Some(repo_name) = args.github_repo {
        println!("☁️ Creating GitHub repo: {}", repo_name);
        create_github_repo(&repo_name).await;
        let username = get_git_username().await;
        run("git", &["branch", "-M", "main"])
            .await
            .expect("Failed to rename branch to main");
        run(
            "git",
            &[
                "remote",
                "add",
                "origin",
                &format!("https://github.com/{}/{}.git", username, repo_name),
            ],
        )
        .await
        .expect("Failed to add remote origin");
        run("git", &["push", "-u", "origin", "main"])
            .await
            .expect("Failed to push to remote repository");
    }

    println!("✅ Setup Completed 🐍");
}

async fn run(cmd: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new(cmd).args(args).status().await?;
    if !status.success() {
        Err(format!("❌ Failed to run: {} {:?}", cmd, args).into())
    } else {
        Ok(())
    }
}

async fn download_file(url: &str, filename: &str) {
    let body = reqwest::get(url).await.unwrap().text().await.unwrap();
    fs::write(filename, body).await.unwrap();
}

async fn create_github_repo(name: &str) {
    let token = env::var("GITHUB_TOKEN").expect("Set GITHUB_TOKEN env var");
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.github.com/user/repos")
        .bearer_auth(token)
        .header("User-Agent", "pycargo")
        .json(&serde_json::json!({ "name": name }))
        .send()
        .await
        .unwrap();

    if !res.status().is_success() {
        panic!(
            "❌ Failed to create GitHub repo: {:?}",
            res.text().await.unwrap()
        );
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
