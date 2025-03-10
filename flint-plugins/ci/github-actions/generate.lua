function Generate(config)
    local workflow = {
        name = "Flint CI",
        on = {
            push = { branches = { "main" } },
            pull_request = { branches = { "main" } }
        },
        jobs = {
            flint_checks = {
                name = "Flint Checks",
                ["runs-on"] = "ubuntu-latest",
                steps = {
                    {
                        name = "Checkout code",
                        uses = "actions/checkout@v3"
                    },
                    {
                        name = "Install Dependencies",
                        run = [[
                            sudo apt-get update
                            sudo apt-get install -y git curl build-essential
                            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
                            source "$HOME/.cargo/env"
                        ]]
                    },
                    {
                        name = "Install Flint",
                        run = [[
                            git clone https://github.com/skadewdl3/flint
                            cd flint
                            cargo build --release
                            cp target/release/flint ../flint
                            echo "$PWD" >> $GITHUB_PATH
                            cd ..
                            chmod +x ./flint
                        ]]
                    },
                    {
                        name = "Run Tests",
                        run = [[
                            ./flint test --test
                            echo "Processing test results..."
                        ]]
                    },
                    {
                        name = "Upload Test Results",
                        uses = "actions/upload-artifact@v3",
                        with = {
                            name = ".flint/reports/reports/report.json",
                            path = ".flint/reports/reports/report.json"
                        }
                    }
                }
            }
        }
    }

    return {
        ["workflows.yml"] = yaml.stringify(workflow)
    }
end
