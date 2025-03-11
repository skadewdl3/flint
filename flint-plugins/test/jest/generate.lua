local log = require("log")
local path = require("path")
local json = require("json")
function Generate(config)
    local common = config
    log.info("Generating Jest configuration")


    local cwd = path.cwd()

    -- Import necessary modules
    local imports = {
        jest = "jest"
    }

    -- Jest Configurations
    local function getTestFolders(test_folders)
        if not test_folders or #test_folders == 0 then return nil end
        return test_folders
    end

    local function getTestMatch(files_include)
        if not files_include or #files_include == 0 then return { "**/__tests__/**/*.js", "**/?(*.)+(spec|test).js" } end
        return files_include
    end

    local function getTestIgnore(files_ignore)
        if not files_ignore or #files_ignore == 0 then return { "/node_modules/", "/dist/" } end
        return files_ignore
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
    local jestConfig = {
        testEnvironment = getTestEnvironment(common.test_environment),
        verbose = getVerboseConfig(common.verbose),
        collectCoverage = getCoverageConfig(common.collect_coverage),
        testMatch = getTestMatch(common.test_files_include),
        testPathIgnorePatterns = getTestIgnore(common.test_files_ignore),
    }


    -- Convert the table to a JSON string
    return {
        ["jest.config.js"] =
            "export default " .. json.stringify(jestConfig) .. ";"
    }
end
