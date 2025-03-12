local log = require("log")
local json = require("json")
local js = require("js")


local function getIndentRule(indent_config)
    if not indent_config then return nil end

    local indent_value = indent_config.indent_style == "spaces" and indent_config.indent_size or "tab"

    local options = js.object({
        SwitchCase = indent_config.switch_case,
        FunctionDeclaration = js.object({
            body = indent_config.function_declaration.body,
            parameters = indent_config.function_declaration.parameters
        }),
        ignoredNodes = indent_config.ignore_nodes,
        ignoreComments = indent_config.ignore_comments,
        VariableDeclarator = indent_config.variable_declarator,
        MemberExpression = indent_config.member_expression,
        FunctionExpression = js.object({
            body = indent_config.function_expression and indent_config.function_expression.body,
            parameters = indent_config.function_expression and indent_config.function_expression.parameters
        }),
        CallExpression = js.object({
            arguments = indent_config.call_expression and indent_config.call_expression.arguments
        }),
        ArrayExpression = indent_config.array_expression,
        ObjectExpression = indent_config.object_expression,
        ImportDeclaration = indent_config.import_declaration,
        flatTernaryExpressions = indent_config.flat_ternary_expressions,
        offsetTernaryExpressions = indent_config.offset_ternary_expressions
    })

    -- No need to remove nil values, the js.object handles that automatically

    return {
        name = "indent",
        value = indent_value,
        options = options
    }
end

local function getQuotesRule(quote_style)
    if not quote_style then return nil end

    local options = js.object({
        avoidEscape = true,
        allowTemplateLiterals = "always"
    })

    return {
        name = "quotes",
        value = quote_style,
        options = options
    }
end

local function getSemiRule(require_semicolons)
    if require_semicolons == nil then return nil end

    local semi_value = require_semicolons and "always" or "never"

    return {
        name = "semi",
        value = semi_value
    }
end

function Generate(config)
    local common = config.common
    local extra = config.config
    log.info("Generating ESLint configuration")

    -- Import necessary plugins
    local stylistic = js.imports.default("stylistic", '@stylistic/eslint-plugin')
    local stylisticJs = js.imports.default("stylisticJs", '@stylistic/eslint-plugin-js')
    local stylisticTs = js.imports.default("stylisticTs", '@stylistic/eslint-plugin-ts')
    local stylisticJsx = js.imports.default("stylisticJsx", '@stylistic/eslint-plugin-jsx')
    local defineConfig = js.imports.named("defineConfig", "eslint/config")
    local recommended = js.imports.default("js", "@eslint/js")
    local globalIgnores = js.imports.named("globalIgnores", "eslint/config")

    local imports = js.imports.merge(stylistic, stylisticJs, stylisticTs, stylisticJsx, defineConfig, globalIgnores,
        recommended)
    imports = tostring(imports)


    local eslintConfig = js.object({})

    -- Generate plugins section as a js object
    local plugins = js.object({
        ["@stylistic"] = stylistic,
        ["@stylistic/js"] = stylisticJs,
        ["@stylistic/ts"] = stylisticTs,
        ["@stylistic/jsx"] = stylisticJsx
    })
    eslintConfig.plugins = plugins

    local rules = js.object({})

    -- Add each rule that has a value
    local indentRule = getIndentRule(common.indent)
    if indentRule then
        rules["@stylistic/" .. indentRule.name] = js.array("error", indentRule.value, indentRule.options)
    end

    local quotesRule = getQuotesRule(common.quote_style)
    if quotesRule then
        rules["@stylistic/" .. quotesRule.name] = js.array("error", quotesRule.value, quotesRule.options)
    end

    local semiRule = getSemiRule(common.require_semicolons)
    if semiRule then
        rules["@stylistic/" .. semiRule.name] = js.array("error", semiRule.value)
    end

    -- Add rules table to eslint config
    eslintConfig.rules = rules

    local ignoresAndIncludes = js.object({})

    if extra.include then
        ignoresAndIncludes.files = extra.include
    end

    if extra.exclude then
        ignoresAndIncludes.ignores = extra.exclude
    end



    eslintConfig = js.exports.default(
        js.fn.call(defineConfig, js.array(
            ignoresAndIncludes,
            js.object({
                plugins = { js = recommended },
                extends = { "js/recommended" }
            }),
            eslintConfig
        ))
    )
    eslintConfig = js.indent(eslintConfig)



    return {
        ["eslint.config.js"] = imports ..
            "\n\n" ..
            eslintConfig
    }
end
