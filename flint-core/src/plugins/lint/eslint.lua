function Details()
    local details = {
        id = "eslint",
        author = "Soham Karandikar",
        version = "0.0.1",
        extensions = { "js", "ts", "mjs", "cjs" },
        category = "lint"
    }
    return details
end

function Validate(config)
    -- sleep(2000)
    local temp = { foo = 'bar', baz = 123, qux = { nested = true } }
    log.debug(config);
    return true
end

function Generate(config)
    local common = config.common
    log.info("Generating ESLint configuration")
    -- log.debug(common)

    --Import necessary plugins
    local imports = {
        stylistic = '@stylistic/eslint-plugin',
        stylisticJs = '@stylistic/eslint-plugin-js'
    }
    local plugins = {
        ["@stylistic"] = "stylistic",
        ["@stylistic/js"] = "stylisticJs"
    }

    
    --Function to get correct stylistic plugin based on rule extension
    local function getplugin(rule_key)
        local extension_map = {
            js = "stylisticJs",
            ts = "stylisticTs",
        }
        local lang_ext = rule_key:match("_(%a+)$") -- Extract '_js' or '_ts'
        if lang_ext and extension_map[lang_ext] then
            local plugin_name = extension_map[lang_ext]
            if not imports[plugin_name] then
                imports[plugin_name] = "@stylistic/eslint-plugin-".. lang_ext
                plugins["@stylistic/" .. lang_ext] = plugin_name
            end
            return "@stylistic/" .. lang_ext
        end
        return "@stylistic"
    end
    
    local function createRuleConfig(rule_name, value, options)
        if value == nil then return {} end 

        local base_config = { "error", value }
        if options then
            table.insert(base_config, options)
        end

        return {
            [rule_name] = base_config,
            ['@stylistic/' .. rule_name] = base_config
        }
    end
    
    local function getIndentRule(indent_config)
        if not indent_config then return nil end
        
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
        
        -- Clean nil values
        for k, v in pairs(options) do
            if v == nil then options[k] = nil end
        end
        
        return createRuleConfig(
            "indent",
            indent_config.indent_style == "spaces" and indent_config.indent_size or "tab",
            options
        )
    end

    local function getQuotesRule(quote_style)
        return createRuleConfig(
            "quotes",
            quote_style,
            {
                avoidEscape = true,
                allowTemplateLiterals = true
            }
        )
    end

    local function getSemiRule(require_semicolons)
        return createRuleConfig("semi", require_semicolons)
    end

    local function getCommaDangleRule(comma_dangle)
        return createRuleConfig(
            "comma-dangle",
            comma_dangle,
            {
                arrays = "always-multiline",
                objects = "always-multiline",
                imports = "always-multiline",
                exports = "always-multiline",
                functions = "never"
            }
        )
    end

    local function getSpaceBeforeFunctionParenRule(space_before_fn_paren)
        local options = {
            anonymous = "always",
            named = "never",
            asyncArrow = "always"
        }
        return createRuleConfig("space-before-function-paren", space_before_fn_paren, options)
    end

    local function getObjectCurlySpacingRule(object_curly_spacing)
        return createRuleConfig(
            "object-curly-spacing",
            object_curly_spacing,
            {
                arraysInObjects = true,
                objectsInObjects = true
            }
        )
    end

    local function getArrowSpacingRule(arrow_spacing)
        return createRuleConfig(
            "arrow-spacing",
            arrow_spacing,
            {
                before = true,
                after = true
            }
        )
    end

    local function getKeySpacingRule(key_spacing)
        return createRuleConfig(
            "key-spacing",
            key_spacing,
            {
                beforeColon = false,
                afterColon = true,
                mode = "strict"
            }
        )
    end

    local function getLinebreakStyleRule(linebreak_style)
        return createRuleConfig("linebreak-style", linebreak_style)
    end

    --Function to map Flint rules to ESLint rules
    local function mapFlintToESLint(flint_key, eslint_rule)
        local value = common[flint_key]
        if value ~= nil then
            local plugin = getplugin(flint_key)
            return {
                [eslint_rule] = { "error", value },
                [plugin .. "/" .. eslint_rule] = { "error", value }
            }
        end
        return {}
    end


    --Define ESLint rules
    local rules = {
        indent = getIndentRule(common.indent),
        quotes = getQuotesRule(common.quote_style),
        semi = getSemiRule(common.require_semicolons),
        ["comma-dangle"] = getCommaDangleRule(common.comma_dangle),
        ["no-trailing-spaces"] = createRuleConfig("no-trailing-spaces", common.trailing_whitespace),
        ["space-before-function-paren"] = getSpaceBeforeFunctionParenRule(common.space_before_function_paren),
        ["object-curly-spacing"] = getObjectCurlySpacingRule(common.object_curly_spacing),
        ["arrow-spacing"] = getArrowSpacingRule(common.arrow_spacing),
        ["key-spacing"] = getKeySpacingRule(common.key_spacing),
        ["linebreak-style"] = getLinebreakStyleRule(common.linebreak_style)
    }
    -- Remove nil values
    for key, value in pairs(rules) do
        if value == nil then rules[key] = nil end
    end

    -- Prepare the ESLint config as a Lua table
    local eslintConfig = {
        {
            plugins = plugins,
            rules = rules
        }
    }

    -- Generate Import statements 
    local importStatements = ""
    for name, path in pairs(imports) do
        importStatements = importStatements .. "import " .. name .. " from '" .. path .. "';\n"
    end

    -- Convert the table to a JSON string
    return {
        ["eslint.config.js"] = importStatements ..
            "\nexport default " .. to_json(eslintConfig) .. ";"
    }
end