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
                        name = "Install Git",
                        run = [[
                        sudo apt-get update
                        sudo apt-get install -y git
                        ]]
                    },
                    {
                        name = "Install Node.js",
                        uses = "actions/setup-node@v3",
                        with = {
                            ["node-version"] = "18.x"
                        }
                    },
                    {
                        name = "Install Jest and ESLint",
                        run = [[
                                                npm install -g jest eslint --no-fund --no-audit --silent
                                                echo "Node.js, Jest, and ESLint installed successfully"
                                            ]]
                    },
                    {
                        name = "Install Flint",
                        uses = "robinraju/release-downloader@v1",
                        with = {
                            latest = true,
                            repository = "skadewdl3/flint",
                            ["out-file-path"] = ".",
                            fileName = "flint",
                            prerelease = true
                        }
                    },
                    {
                        name = "Run Tests",
                        run = [[
                            chmod +x ./flint
                            ./flint install
                            ./flint test --test
                        ]]
                    },
                    {
                        name = "Upload Test Results",
                        uses = "actions/upload-artifact@v4",
                        with = {
                            name = "Test Results",
                            path = ".flint/reports/reports/report.json"
                        }
                    },
                    {
                        name = "Upload Logs",
                        uses = "actions/upload-artifact@v4",
                        with = {
                            name = "Logs",
                            path = "logs.txt",
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
