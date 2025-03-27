# Flint

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Flint is a software tool designed to automate and standardize code linting, testing, and CI/CD configuration across diverse software projects.

## Overview

Many software projects struggle with inconsistent code quality, unreliable testing practices, and complex CI/CD setups, leading to increased development time, higher error rates, and reduced overall efficiency. Flint addresses these challenges by providing a centralized platform for managing code quality rules, automating tests, and generating configuration files for various CI/CD systems. It uses a plugin-based architecture, allowing for easy customization and extension to support different languages, frameworks, and tools.

## Goals and Objectives

Flint aims to achieve the following goals:

*   **Improve Code Quality:** Enforce consistent coding standards and best practices across projects through automated linting.
*   **Automate Testing:** Streamline and automate the testing process to ensure code reliability and reduce the risk of errors.
*   **Simplify CI/CD Configuration:** Generate configuration files for various CI/CD systems, making it easier to set up and maintain automated deployment pipelines.
*   **Increase Developer Productivity:** Automate repetitive tasks and provide developers with a consistent and efficient workflow.
*   **Reduce Development Costs:** Minimize the time and resources spent on manual code reviews, testing, and CI/CD configuration.
*   **Support a Wide Range of Languages and Tools:** Offer a flexible plugin-based architecture that can be easily extended to support different programming languages, frameworks, and development tools.

## Features

*   **Plugin-Based Architecture:** Easily extend Flint's functionality by installing plugins for different languages, linters, and testing frameworks.
*   **Automated Linting:** Automatically identify and report code quality issues based on configurable rules.
*   **Automated Testing:** Run tests automatically and generate reports on test results.
*   **CI/CD Configuration Generation:** Generate configuration files for popular CI/CD systems like GitHub Actions, GitLab CI, and Jenkins.
*   **TUI (Textual User Interface):** Provides an interactive TUI for configuring and running Flint.
*   **Dependency Management:** Manages plugin dependencies to ensure compatibility and avoid conflicts.
*   **Reporting:** Generates reports on code quality, test results, and other metrics.

## Getting Started

1.  **Download the latest release:** Obtain the latest pre-built binary from the [Releases page](link-to-releases-page). Also make sure you have **Git** installed on your system, since Flint requires Git to download plugins.
2.  **Make Flint executable:** Open your terminal and navigate to the directory where you downloaded the Flint binary. Then, run the following command to make it executable:

    ```bash
    chmod +x ./flint
    ```

3.  **Initialize Flint:** Create a `flint.toml` configuration file by running the following command:

    ```bash
    ./flint init
    ```

4.  **Configure `flint.toml`:**  Edit the `flint.toml` file to configure Flint for your project.  You can use the following as a starting point (replace the contents of your `flint.toml` with this):

    ```toml
    [flint]
    version = 1
    plugins_branch = "final-stretch"
    env = ".env"

    [rules.common]
    # Basic style settings
    max_line_length = 100
    quote_style = "double"
    require_semicolons = true
    indent_style = "spaces"
    indent_size = 4

    # Whitespace rules
    no_trailing_spaces = true
    no_multiple_empty_lines = true

    [rules.eslint]

    [config.eslint]
    exclude = ["*.config.js", "coverage/**/*", "node_modules/**/*"]
    include = ["src/**/*.js"]

    [tests.jest]
    enabled = true
    config_path = "src/jest.config.ts"
    test_environment = "node"
    verbose = true
    collect_coverage = true
    include = ["src/*.test.js"]
    exclude = ["node_modules/", "dist/"]


    [ci.github-actions]
    artifacts = "reports"


    [report.db.env]
    host = "env:MYSQL_HOST"
    port = "env:MYSQL_PORT"
    username = "env:MYSQL_USERNAME"
    password = "env:MYSQL_PASSWORD"
    database = "env:MYSQL_DATABASE"

    [report.json]
    output_path = "reports"

    [report.ai]
    output_path = "reports"
    [report.ai.env]
    API_BASE_URL = "env:GEMINI_BASE_URL"
    API_KEY = "env:GEMINI_API_KEY"
    MODEL = "gemini-2.0-flash"
    ```

    **Note:**  Make sure to adjust the `flint.toml` file to match your project's specific needs.  Pay particular attention to the `include` and `exclude` settings, as well as the plugin configurations. Also, some plugins expect environment variables to be set. The report plugins, for example.

5.  **Install Plugins:** Install the plugins specified in your `flint.toml` file:

    ```bash
    ./flint install
    ```

    **Note:** This step requires an internet connection to download the plugins. This will access the `skadewdl3/flint` GitHub Repo to download plugins.


6.  **Generate Configuration Files:** Generate configuration files for the plugins you have configured in `flint.toml`:

    ```bash
    ./flint generate
    ```
For instance, if you have configured the `eslint` plugin, Flint will generate an `eslint.config.js` based on your configuration.


7.  **Run Tests:** Run your project's tests using Flint:

    ```bash
    ./flint test
    ```

    **Note:** The output and behavior will depend on the plugins you have installed and configured.

## Usage

Flint is configured using a `flint.toml` file located in the root of your project. This file defines the plugins to use, their configurations, and other settings for your project.

### Understanding the `flint.toml` File

The `flint.toml` file is structured into several sections:

*   **`[flint]`:** This section contains global Flint settings:

    *   `version = 1`: Specifies the version of the Flint configuration format.
    *   `plugins_branch = "main"`:  Specifies the branch of the Flint plugin repository to use.  This allows you to select different versions of the plugins.
    *   `env = ".env"` (Optional):  Specifies the path to a `.env` file containing environment variables that should be made available to the plugins.  This is useful for storing sensitive information or project-specific settings.

*   **`[rules.plugin_id]` (Lint Plugins):**  This section configures linting plugins. `plugin_id` is the ID of the plugin (e.g., `eslint`).

    *   Example:

        ```toml
        [rules.eslint]
        # Configuration specific to the eslint plugin goes here
        ```

    *   These settings are plugin specific, and depend on the plugin.

*   **`[config.plugin_id]` (Lint Plugins):** Similar to the `rules` section, this is optional and defines global configurations that might be used for linting. For example, you might want to exclude files from analysis.

    *   Example:

        ```toml
        [config.eslint]
        exclude = ["*.config.js", "coverage/**/*", "node_modules/**/*"]
        include = ["src/**/*.js"]
        ```

*   **`[tests.plugin_id]` (Test Plugins):**  This section configures testing plugins. `plugin_id` is the ID of the plugin (e.g., `jest`).

    *   Example:

        ```toml
        [tests.jest]
        enabled = true
        config_path = "src/jest.config.ts"
        test_environment = "node"
        verbose = true
        collect_coverage = true
        include = ["src/*.test.js"]
        exclude = ["node_modules/", "dist/"]
        ```

    *   These settings are plugin specific, and depend on the plugin.

*   **`[ci.plugin_id]` (CI Plugins):**  This section configures Continuous Integration (CI) plugins.  `plugin_id` is the ID of the plugin (e.g., `github-actions`).

    *   Example:

        ```toml
        [ci.github-actions]
        artifacts = "reports"
        ```

    *   These settings are plugin specific, and depend on the plugin.

*   **`[report.plugin_id]` (Report Plugins):**  This section configures report plugins.  `plugin_id` is the ID of the plugin (e.g., `json`, `ai`).

    *   Example:

        ```toml
        [report.json]
        output_path = "reports"

        [report.ai]
        output_path = "reports"
        [report.ai.env]
        API_BASE_URL = "env:GEMINI_BASE_URL"
        API_KEY = "env:GEMINI_API_KEY"
        MODEL = "gemini-2.0-flash"
        ```

    *   These settings are plugin specific, and depend on the plugin.  Note the use of `env:` to specify environment variables.

### Flint Commands and Options

Flint provides the following commands:

*   **`init`:** Initializes a new `flint.toml` file in the current directory.

    *   `./flint init`: Creates a default `flint.toml` file.
    *   This command does not take options.

*   **`install`:** Installs the plugins specified in the `flint.toml` file.

    *   `./flint install`: Installs all plugins listed in the `flint.toml`.
    *   **Options:**
        *   `--lint`: Install all linting plugins.
        *   `--test`: Install all testing plugins.
        *   `--all`:  Install all plugins (this is the default behavior).
        *   `--help`: Show help for the install command.

    *   Example: `./flint install --lint` will install just linting plugins

*   **`generate`:** Generates configuration files for the configured plugins (e.g., `.eslintrc.js`, `jest.config.js`).  This command is useful for setting up the linters and testing frameworks in your project.

    *   `./flint generate`: Generates configuration files for all configured plugins.
    *   **Options:**
        *   `-h, --help`: Show help for the generate command.

*   **`test`:** Runs the configured tests and linters.

    *   `./flint test`: Runs all configured tests and linters.
    *   **Options:**
        *   `-h, --help`: Show help for the test command.
        *   `-a, --all`: Runs all tests. This is the default option if you do not specify any filters.
        *   `-l, --lint`: Runs only the linting plugins.
        *   `-t, --test`: Runs only the testing plugins.

    *   Example: `./flint test --lint` will run only linting plugins.

*   **`help`:** Displays help information about Flint and its commands.

    *   `./flint help`: Shows the general help message.  (Note: Flint's help functionality is currently limited.)
    *   This command does not take options.

## Contributing

We welcome contributions to Flint! I don't really have a `CONTRIBUTING.md` file yet, but I'd be more than happy to help you out if you need any help. If this project some, if any, traction, I'll add a `CONTRIBUTING.md` file to the repository.

For the basics, follow these steps to start contributing:
1. Install [Rust](https://www.rust-lang.org/learn/get-started)
2. Install [Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
3. Clone this repo. Then, clone its submodules using `git submodule update --init --recursive`.
3. If your'e modifying the core functionality of Flint, you'll need to work with the `flint` and `flint-ffi` crates.
4. If you're contributing a new plugin, you can use the plugin template from any of the plugins in the `flint-plugins` folder.
5. You can use the `run.sh` file included in this repo to easily test Flint. This is recommended so because Flint tries to install plugins if it can't find them in the user data directory every time it is run. To prevent this, the `--no-install` flag must be passed along with the `--plugins-dir` flag.
6. `flint-utils` contains some utility functions to be used with Flint. This mostly involves custom error types, functions to load config files, read/set env variables, etc.
7. The `flint-macros` crate contains two macros - `widget!()` and `ui!()`. These are used to simplify writing Ratatui UI widgets. The macros are mostly complete, and work as expected pretty much everywhere. If you're working with the UI, I recommend you use them.

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/skadewdl3/flint/blob/main/LICENSE) file for details.

## Future Plans

*   **Detailed Installation and Usage Instructions:**  Provide comprehensive documentation on how to install and use Flint.
*   **More Plugins:** Develop and integrate more plugins for different languages, frameworks, and tools.
*   **Improved Reporting:** Enhance the reporting capabilities to provide more detailed and actionable insights.
*   **Web UI:** Develop a web-based user interface for configuring and running Flint.

## Team

*   Soham Karandikar ([https://github.com/skadewdl3]) - Original Author and Maintainer

## Acknowledgments

*   This project uses [ratatui](https://github.com/ratatui-org/ratatui), a Rust library for building TUIs.
*   This project uses [mlua](https://github.com/khvzak/mlua), a Rust library for embedding Lua.
* This project uses multiple open-source libraries which can be found in `Cargo.toml` files for all members in this Cargo workspace. In case there are any licensing conflicts, please let me know. I'll work on fixing them ASAP.
