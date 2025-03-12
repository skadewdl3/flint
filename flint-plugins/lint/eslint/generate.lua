local log = require("log")
local json = require("json")

function Generate(config)
    local common = config.common
    local extra = config.config
    log.info("Generating ESLint configuration")

    -- Import necessary plugins
    local imports = {
        stylistic = '@stylistic/eslint-plugin',
        stylisticJs = '@stylistic/eslint-plugin-js',
        stylisticTs = '@stylistic/eslint-plugin-ts',
        stylisticJsx = '@stylistic/eslint-plugin-jsx',
    }

    local plugins = {
        ["@stylistic/jsx"] = "stylisticJsx",
        ["@stylistic/ts"] = "stylisticTs",
        ["@stylistic/js"] = "stylisticJs",
        ["@stylistic"] = "stylistic"
    }

    -- Generate plugins section as a formatted string
    local pluginsSection = "{\n"
    for pluginKey, pluginVar in pairs(plugins) do
        pluginsSection = pluginsSection .. '    "' .. pluginKey .. '": ' .. pluginVar .. ',\n'
    end
    pluginsSection = pluginsSection .. "  }"

    -- Helper function to format options object
    local function formatOptions(options)
        if type(options) ~= "table" then return "{}" end

        local result = "{\n"
        for k, v in pairs(options) do
            if v ~= nil then
                result = result .. '        "' .. k .. '": '

                if type(v) == "string" then
                    result = result .. '"' .. v .. '"'
                elseif type(v) == "number" then
                    result = result .. v
                elseif type(v) == "boolean" then
                    result = result .. tostring(v)
                elseif type(v) == "table" then
                    result = result .. formatOptions(v)
                end

                result = result .. ",\n"
            end
        end
        result = result .. "      }"

        return result
    end

    -- Function to format rule entries directly as strings
    local function formatRuleEntry(rule_name, value, options)
        if value == nil then return nil end

        local rule_str = '    "@stylistic/' .. rule_name .. '": [\n'
        rule_str = rule_str .. '      "error",\n'

        if type(value) == "string" then
            rule_str = rule_str .. '      "' .. value .. '"'
        elseif type(value) == "number" then
            rule_str = rule_str .. '      ' .. value
        elseif type(value) == "boolean" then
            if value then
                rule_str = rule_str .. '      "always"'
            else
                rule_str = rule_str .. '      "never"'
            end
        else
            rule_str = rule_str .. '      ' .. tostring(value)
        end

        if options then
            rule_str = rule_str .. ',\n      ' .. formatOptions(options) .. '\n'
        else
            rule_str = rule_str .. '\n'
        end

        rule_str = rule_str .. '    ]'

        return rule_str
    end

    -- Process specific rules from flint.toml
    local function getIndentRule(indent_config)
        if not indent_config then return nil end

        local indent_value = indent_config.indent_style == "spaces" and indent_config.indent_size or "tab"

        local options = {
            SwitchCase = indent_config.switch_case,
            FunctionDeclaration = {
                body = indent_config.function_declaration.body,
                parameters = indent_config.function_declaration.parameters
            },
            ignoredNodes = indent_config.ignore_nodes,
            ignoreComments = indent_config.ignore_comments,
            VariableDeclarator = indent_config.variable_declarator,
            MemberExpression = indent_config.member_expression,
            FunctionExpression = {
                body = indent_config.function_expression and indent_config.function_expression.body,
                parameters = indent_config.function_expression and indent_config.function_expression.parameters
            },
            CallExpression = {
                arguments = indent_config.call_expression and indent_config.call_expression.arguments
            },
            ArrayExpression = indent_config.array_expression,
            ObjectExpression = indent_config.object_expression,
            ImportDeclaration = indent_config.import_declaration,
            flatTernaryExpressions = indent_config.flat_ternary_expressions,
            offsetTernaryExpressions = indent_config.offset_ternary_expressions
        }

        -- Remove nil values
        for k, v in pairs(options) do
            if v == nil then options[k] = nil end
        end

        return formatRuleEntry("indent", indent_value, options)
    end

    local function getQuotesRule(quote_style)
        if not quote_style then return nil end

        local options = {
            avoidEscape = true,
            allowTemplateLiterals = "always"
        }

        return formatRuleEntry("quotes", quote_style, options)
    end

    local function getSemiRule(require_semicolons)
        if require_semicolons == nil then return nil end

        local semi_value = require_semicolons and "always" or "never"
        return formatRuleEntry("semi", semi_value)
    end

    local function getTrailingSpacesRule(trailing_whitespace)
        if trailing_whitespace == nil then return nil end

        return formatRuleEntry("no-trailing-spaces", trailing_whitespace)
    end

    -- Build the rules section string
    local rulesString = "{\n"

    -- Add each rule that has a value
    local indentRule = getIndentRule(common.indent)
    if indentRule then
        rulesString = rulesString .. indentRule .. ",\n"
    end

    local quotesRule = getQuotesRule(common.quote_style)
    if quotesRule then
        rulesString = rulesString .. quotesRule .. ",\n"
    end

    local semiRule = getSemiRule(common.require_semicolons)
    if semiRule then
        rulesString = rulesString .. semiRule .. ",\n"
    end


    -- Close the rules object
    rulesString = rulesString .. "}"

    -- Generate import statements
    local importStatements = ""
    for name, path in pairs(imports) do
        importStatements = importStatements .. "import " .. name .. " from '" .. path .. "';\n"
    end

    -- Build the final config file
    local eslintConfig = importStatements ..
        "\nexport default [{\n" ..
        "  plugins: " .. pluginsSection .. ",\n" ..
        "  rules: " .. rulesString .. "\n" ..
        "}];"

    return {
        ["eslint.config.js"] = eslintConfig
    }
end
