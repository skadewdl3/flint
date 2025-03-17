local log = require("log")
local path = require("path")
local json = require("json")
local js = require("js")

function Generate(config)
    local common = config

    -- Jest Configurations

    local function getRootDir(root_dir)
        if not root_dir then return "." end
        return root_dir
    end

    local function getTestMatch(include)
        if not include or #include == 0 then return { "**/__tests__/**/*.js", "**/?(*.)+(spec|test).js" } end
        local paths_with_root_dir = {}
        for i, file in ipairs(include) do
            paths_with_root_dir[i] = path.join("<rootDir>", file)
        end
        return paths_with_root_dir
    end

    local function getTestIgnore(exclude)
        if not exclude or #exclude == 0 then return { "/node_modules/", "/dist/" } end
        return exclude
    end

    local function getCoverageConfig(coverage)
        return coverage or false
    end

    local function getVerboseConfig(verbose)
        return verbose or true
    end

    local function getTestEnvironment(env)
        return env or "node"
    end

    -- Define Jest configuration
    local jestConfig = js.object({
        testEnvironment = getTestEnvironment(common.test_environment),
        verbose = getVerboseConfig(common.verbose),
        collectCoverage = getCoverageConfig(common.collect_coverage),
        testMatch = getTestMatch(common.include),
        testPathIgnorePatterns = getTestIgnore(common.exclude),
        rootDir = getRootDir(common.root_dir)
    })

    jestConfig = js.exports.default(jestConfig)
    jestConfig = js.indent(jestConfig)


    -- Convert the table to a JSON string
    return {
        ["jest.config.js"] = jestConfig
    }
end
