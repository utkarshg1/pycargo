# PyCargo

PyCargo is a CLI tool designed to bootstrap Python projects with ease. It helps you set up a Python project directory, manage dependencies, initialize Git repositories, and optionally create a GitHub repository.

## Features

- Create a Python project directory.
- Generate a `requirements.txt` file based on predefined setups (`basic`, `advanced`, `data-science`, or `blank`).
- Initialize a Git repository and configure Git settings.
- Create a GitHub repository (requires a GitHub Personal Access Token).
- Automatically install and configure `uv` for virtual environment and dependency management.

## Installation

Download the latest release from the `wix/pycargo-0.2.0-x86_64.msi` file and install it on your system.

## Usage

Run the `pycargo` command with the desired options:

### Basic Usage

```bash
pycargo my_project
```

This creates a project directory named `my_project` with the default `advanced` setup.

### Specify a Setup Type

```bash
pycargo my_project --setup basic
```

Available setup types:

- `basic`: Installs `numpy`, `pandas`, `matplotlib`, `seaborn`, and `ipykernel`.
- `advanced`: Installs additional libraries like `plotly`, `requests`, and `streamlit`.
- `data-science`: Includes `basic` libraries plus `scikit-learn`, `statsmodels`, `streamlit`, and `xgboost`.
- `blank`: Creates an empty `requirements.txt` file for custom dependencies.

### Create a GitHub Repository (Public by Default)

```bash
pycargo my_project --github-repo my_project_repo
```

This creates a public GitHub repository named `my_project_repo` and links it to the local Git repository.

### Create a Private GitHub Repository

```bash
pycargo my_project --github-repo my_project_repo --private
```

This creates a private GitHub repository named `my_project_repo` and links it to the local Git repository.

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

```bash
pycargo data_project --setup data-science --github-repo data_project_repo
```

This creates a `data_project` directory, sets up a `data-science` environment, initializes a Git repository, and creates a linked GitHub repository named `data_project_repo`.

## License

This project is licensed under the Apache License 2.0. See the `LICENSE` file for details.
