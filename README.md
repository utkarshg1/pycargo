# PyCargo

PyCargo is a CLI tool designed to bootstrap Python projects with ease. It helps you set up a Python project directory, manage dependencies, initialize Git repositories, and optionally create a GitHub repository.

## Features

- Create a Python project directory.
- Generate a `requirements.txt` file based on predefined setups (`basic`, `advanced`, `data-science`, or `blank`).
- Initialize a Git repository and configure Git settings.
- Automatically check and set Git configuration (`user.name` and `user.email`).
- Create a GitHub repository (requires a GitHub Personal Access Token).
- Specify whether the GitHub repository should be private or public.
- Automatically install and configure `uv` for virtual environment and dependency management.
- Download essential files like `.gitignore` and `LICENSE` automatically from predefined URLs.

## Installation

Download the latest release from the `pycargo-0.2.10-x86_64.msi` file and install it on your system.

## Usage

Run the `pycargo` command with the desired options:

### Demo

![pycargo_demo](./demo/pycargo_demo.gif)

### Basic Usage

```cmd
pycargo -n my_project
```

This creates a project directory named `my_project` with the default `advanced` setup.

### Specify a Setup Type

```cmd
pycargo -n my_project -s basic
```

Available setup types:

- `basic`: Installs `numpy`, `pandas`, `matplotlib`, `seaborn`, and `ipykernel`.
- `advanced`: Installs additional libraries like `plotly`, `requests`, and `streamlit`.
- `data-science`: Includes `basic` libraries plus `scikit-learn`, `statsmodels`, `streamlit`, and `xgboost`.
- `blank`: Creates an empty `requirements.txt` file for custom dependencies.

### Create a GitHub Repository (Public by Default)

```cmd
pycargo -n my_project -g
```

This creates a public GitHub repository named `my_project` and links it to the local Git repository.

### Create a Private GitHub Repository

```cmd
pycargo -n my_project -g -p
```

This creates a private GitHub repository named `my_project` and links it to the local Git repository.

### Specify a Custom GitHub Repository Name

```cmd
pycargo -n my_project -g --github-repo-name custom_repo_name
```

This creates a GitHub repository named `custom_repo_name` and links it to the local Git repository.

### Activate the Virtual Environment

After the setup is complete, activate the virtual environment:

```cmd
.venv\Scripts\activate
```

### Display Help

To display the help menu with all available options:

```cmd
pycargo --help
```

### Display Version

To display the current version of PyCargo:

```cmd
pycargo -V
```

### Downloaded Files

The following files are automatically downloaded and added to your project:

- `.gitignore`: A standard Python `.gitignore` file from GitHub's official repository.
- `LICENSE`: The Apache License 2.0 from the official Apache website.

### Git Configuration Check

If `user.name` or `user.email` is not set in your Git configuration, PyCargo will prompt you to set them during the setup process.

### `uv` Installation Check

PyCargo ensures that `uv` is installed on your system. If not, it will automatically install it for you.

## Setting Up GitHub Personal Access Token

To create a GitHub repository, you need to set a GitHub Personal Access Token (PAT) as an environment variable:

1. Generate a PAT from your GitHub account with `repo` scope.
2. Set the `GITHUB_TOKEN` environment variable:
   - **Command Prompt**:
     ```cmd
     setx GITHUB_TOKEN "your-token-here"
     ```
   - **PowerShell**:
     ```powershell
     [Environment]::SetEnvironmentVariable("GITHUB_TOKEN", "your-token-here", "User")
     ```
   - **Windows GUI**:
     - Open the Start Menu and search for "Environment Variables".
     - Click on "Edit the system environment variables".
     - Under "User variables", click "New" and set:
       - Variable name: `GITHUB_TOKEN`
       - Variable value: `your-token-here`

## Example

```cmd
pycargo -n data_project -s data-science -g --github-repo-name data_project_repo
```

This creates a `data_project` directory, sets up a `data-science` environment, initializes a Git repository, and creates a linked GitHub repository named `data_project_repo`.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
