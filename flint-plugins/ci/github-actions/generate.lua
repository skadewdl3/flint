local log = require("log")
local yaml = require("yaml")

local function get_dependency_install_steps(dependencies)
    local steps = {}
    -- TODO: Add support for system dependencies

    -- Add support for node.js (npm) dependencies
    if dependencies.npm and #dependencies.npm > 0 then
        -- Add Node.js setup
        table.insert(steps, {
            name = "Install Node.js",
            uses = "actions/setup-node@v3",
            with = {
                ["node-version"] = "18.x"
            }
        })

        -- Install NPM packages
        local npm_deps = {}
        for _, dep in ipairs(dependencies.npm) do
            table.insert(npm_deps, dep.name .. "@" .. dep.version)
        end

        table.insert(steps, {
            name = "Install Node.js dependencies",
            run = "npm install -g " .. table.concat(npm_deps, " ") .. " --no-fund --no-audit --silent"
        })
    end

    -- Add support for Python (pip) dependencies
    if dependencies.pip and #dependencies.pip > 0 then
        -- Add Python setup
        table.insert(steps, {
            name = "Install Python",
            uses = "actions/setup-python@v4",
            with = {
                ["python-version"] = "3.x"
            }
        })

        -- Install pip packages
        local pip_deps = {}
        for _, dep in ipairs(dependencies.pip) do
            table.insert(pip_deps, dep.name .. "==" .. dep.version)
        end

        table.insert(steps, {
            name = "Install Python dependencies",
            run = "pip install " .. table.concat(pip_deps, " ")
        })
    end

    return steps
end

function Generate(config, dependencies)
    log.debug(dependencies)

    local workflow = {}
    workflow.name = "Flint CI"
    workflow.on = {
        pull_request = { branches = { "main" } },
        push = { branches = { "main" } }
    }


    -- Create empty job table
    local job = {
        name = "Flint Checks",
        ["runs-on"] = "ubuntu-latest",
        steps = {}
    }

    -- Checkout code from current repo
    -- Add checkout step
    table.insert(job.steps, {
        name = "Checkout code",
        uses = "actions/checkout@v4"
    })

    local dependency_install_steps = get_dependency_install_steps(dependencies)

    -- Add dependency install steps to the job
    for _, step in ipairs(dependency_install_steps) do
        table.insert(job.steps, step)
    end

    -- install Flint from the latest release
    table.insert(job.steps, {
        name = "Install Flint",
        uses = "robinraju/release-downloader@v1",
        with = {
            latest = true,
            repository = "skadewdl3/flint",
            ["out-file-path"] = ".",
            fileName = "flint",
            prerelease = true
        }
    })

    -- Run Flint Checks
    table.insert(job.steps, {
        name = "Run Tests",
        run =
        [[
        chmod +x ./flint
        ./flint install
        ./flint test --test
        ]]
    })

    table.insert(job.steps, {
        name = "Upload Test Results",
        uses = "actions/upload-artifact@v4",
        -- TODO: Make it return all logs
        -- TODO: Make it adapt to the outputs of reporting plugins
        with = {
            name = "Test Results",
            path = "reports/reports/report.json"
        }
    })

    table.insert(job.steps, {
        name = "Upload Logs",
        uses = "actions/upload-artifact@v4",
        with = {
            name = "Logs",
            path = "logs.txt",
        }
    })

    workflow.jobs = {
        flint_checks = job
    }


    return {
        [".github/workflows/flint_checks.yml"] = yaml.stringify(workflow)
    }
end
