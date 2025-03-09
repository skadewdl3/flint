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
                        name = "Install Flint",
                        run = "cargo install flint"
                    },
                    -- Run linting checks
                    {
                        name = "Run Linting",
                        run = [[
                            # Run linting and save output
                            flint test --lint --json > lint-results.json

                            # Process results and create GitHub annotations
                            echo "Processing lint results..."

                            # Parse JSON and create GitHub annotations
                            jq -r '.issues[] | "::error file=\(.file),line=\(.line),col=\(.column)::\(.message)"' lint-results.json >> $GITHUB_OUTPUT

                            # Save summary statistics
                            echo "### Linting Summary" >> $GITHUB_STEP_SUMMARY
                            jq -r '"Total Issues: \(.total_issues)\nFiles Checked: \(.files_checked)\nSeverity Breakdown:\n  Critical: \(.severity.critical)\n  Major: \(.severity.major)\n  Minor: \(.severity.minor)"' lint-results.json >> $GITHUB_STEP_SUMMARY
                        ]]
                    },
                    -- Run tests
                    {
                        name = "Run Tests",
                        run = [[
                            # Run tests and save output
                            flint test --test --json > test-results.json

                            # Process results and create GitHub annotations
                            echo "Processing test results..."

                            # Parse JSON and create GitHub annotations for failed tests
                            jq -r '.failures[] | "::error file=\(.file),line=\(.line)::\(.test_name): \(.message)"' test-results.json >> $GITHUB_OUTPUT

                            # Save test summary
                            echo "### Test Summary" >> $GITHUB_STEP_SUMMARY
                            jq -r '"Total Tests: \(.total_tests)\nPassed: \(.passed)\nFailed: \(.failed)\nSkipped: \(.skipped)\n\nTest Suites:\n\(.test_suites | to_entries | .[] | "  \(.key): \(.value.passed)/\(.value.total) passed")"' test-results.json >> $GITHUB_STEP_SUMMARY
                        ]]
                    },
                    -- Upload detailed results as artifacts
                    {
                        name = "Upload Test Results",
                        uses = "actions/upload-artifact@v3",
                        with = {
                            name = "test-results",
                            path = "test-results.json"
                        }
                    },
                    {
                        name = "Upload Lint Results",
                        uses = "actions/upload-artifact@v3",
                        with = {
                            name = "lint-results",
                            path = "lint-results.json"
                        }
                    },
                }
            }
        }
    }

    return {
        ["workflows.yml"] = yaml.stringify(workflow)
    }
end
