local log = require("log")
local yaml = require("yaml")
local path = require("path")
local env = require("env")


local function get_dependency_install_steps(dependencies)
    local steps = {}

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


local function get_env_vars(env_vars)
    local function get_env_name(str)
        return string.match(str, "^env:(.+)$") -- Capture everything after "env:"
    end

    local res = {}
    for k, v in pairs(env_vars) do
        local temp = get_env_name(v)
        if temp ~= nil then
            log.debug("${{ secrets." .. temp .. " }}")
            res[temp] = "${{ secrets." .. temp .. " }}"
        else
            res[k] = v
        end
    end

    return res
end

function Generate(config, dependencies, env)
    log.debug(dependencies)

    local workflow = {}
    workflow.name = "Flint CI"
    workflow.on = config.on or {
        pull_request = { branches = { "main" }, types = { "opened", "synchronize", "reopened" } },
        push = { branches = { "main" } }
    }


    -- Create empty job table
    local job = {
        name = "Flint Checks",
        ["runs-on"] = "ubuntu-latest",
        permissions = {
            ["pull-requests"] = "write",
            contents = "read"
        },
        steps = {}
    }

    -- Checkout code from current repo
    -- Add checkout step
    table.insert(job.steps, {
        name = "Checkout code",
        uses = "actions/checkout@v4",
        with = {
            ref = "${{ github.event.pull_request.head.ref || github.ref_ame }}",
            repository = "${{ github.event.pull_request.head.repo.full_name || github.repository }}",
            ["persist-credentials"] = false,
            token = "${{ secrets.GITHUB_TOKEN }}"
        },
    })

    local dependency_install_steps = get_dependency_install_steps(dependencies)
    local env_vars = get_env_vars(env)


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
        },
    })

    -- Run Flint Checks
    env_vars.REPO_BRANCH_URL =
    "${{ github.server_url }}/${{ github.repository }}/blob/${{ github.head_ref || github.ref_name }}"
    table.insert(job.steps, {
        name = "Run Tests",
        run =
        [[
        chmod +x ./flint
        ./flint install
        ./flint test
        ]],
        env = env_vars
    })

    table.insert(job.steps, {
        name = "Comment AI Suggestions",
        run = [[
        if [ -f reports/temp.md ]; then
        gh pr comment ${{ github.event.pull_request.number }} --body "$(cat reports/temp.md)"
        else
        echo "AI suggestions not found, skipping PR comment."
        fi
        ]],
        env = {
            GH_TOKEN = "${{ secrets.GITHUB_TOKEN }} "
        }
    })

    table.insert(job.steps, {
        name = "Upload Test Results",
        uses = "actions/upload-artifact@v4",
        with = {
            name = "Test Results",
            path = config.artifacts
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
