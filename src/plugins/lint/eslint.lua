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
    local imports = {}
    local plugins = {}
    --Default Stylistic Plugin
    imports["stylistic"] = '@stylistic/eslint-plugin'
    plugins["@stylistic"] = "stylistic"
    
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
    

    local function getIndentRule(indent_config)

        if not indent_config then
            return nil
        end

        local indentation = indent_config.indent_style
        local indent_size = indent_config.indent_size
        -- Base indentation configuration
        local indentOptions = {
            "error",
            indentation == "spaces" and indent_size or "tab"
        }

        -- Create options table for the third parameter
        local options = {}

        -- Handle additional indent-specific options if they exist in config
        if indent_config.switch_case then
            options.SwitchCase = indent_config.switch_case
        end
        if indent_config.variable_declarator then
            options.VariableDeclarator = indent_config.variable_declarator
        end
        if indent_config.outer_iife_body then
            options.outerIIFEBody = indent_config.outer_iife_body
        end
        if indent_config.member_expression then
            options.MemberExpression = indent_config.member_expression
        end
        if indent_config.function_declaration then
            options.FunctionDeclaration = {
                body = indent_config.function_declaration.body,
                parameters = indent_config.function_declaration.parameters
            }
        end
        if indent_config.function_expression then
            options.FunctionExpression = {
                body = indent_config.function_expression.body,
                parameters = indent_config.function_expression.parameters
            }
        end
        if indent_config.static_block then
            options.StaticBlock = indent_config.static_block
        end
        if indent_config.call_expression then
            options.CallExpression = {
                arguments = indent_config.call_expression.arguments,
            }
        end
        if indent_config.array_expression then
            options.ArrayExpression = indent_config.array_expression
        end
        if indent_config.object_expression then
            options.ObjectExpression = indent_config.object_expression
        end
        if indent_config.import_declaration then
            options.ImportDeclaration = indent_config.import_declaration
        end
        if indent_config.flat_ternary_expressions ~= nil then
            options.flatTernaryExpressions = indent_config.flat_ternary_expressions
        end
        if indent_config.offset_ternary_expressions ~= nil then
            options.offsetTernaryExpressions = indent_config.offset_ternary_expressions
        end
        if indent_config.ignore_nodes then
            if type(indent_config.ignore_nodes) == "string" then
                options.ignoredNodes = { indent_config.ignore_nodes }
            else
                options.ignoredNodes = indent_config.ignore_nodes
            end
        end
        if indent_config.ignore_comments ~= nil then
            options.ignoreComments = indent_config.ignore_comments
        end

        -- Add the options table as third parameter if any options were set
        if next(options) ~= nil then
            table.insert(indentOptions, options)
        end  
        return indentOptions
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
        semi = mapFlintToESLint("require_semicolons", "semi"),
        quotes = mapFlintToESLint("quote_style", "quotes"),
        ["max-len"] = mapFlintToESLint("max_line_length", "max-len"),
        ["array-bracket-newline"] = mapFlintToESLint("array_bracket_newline_js", "array-bracket-newline"),
        ["array-element-newline"] = mapFlintToESLint("array_element_newline_ts", "array-element-newline"),
        ["no-trailing-spaces"] = mapFlintToESLint("trailing_whitespace", "no-trailing-spaces")
    }

    -- Remove nil values
    for key, value in pairs(rules) do
        if value == nil then rules[key] = nil end
    end

    -- Prepare the ESLint config as a Lua table
    local eslintConfig = {
        {
            plugins = plugins,
            rules = rules,
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
