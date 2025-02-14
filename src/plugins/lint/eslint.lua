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
    log.debug(temp);
    return true
end

function Generate(config)
    local common = config.common
    log.info("Generating ESLint configuration")
    -- log.debug(common)

    local imports = {}
    local plugins = {}

    local function getIndentRule(indent_config)

        if not indent_config then
            return nil
        end

        imports["stylisticJs"] = '@stylistic/eslint-plugin-js'
        plugins["@stylistic/js"] = "stylisticJs"   

        local indentation = common.indent.indent_style
        local indent_size = common.indent.indent_size
        -- Base indentation configuration
        local indentOptions = {
            "error",
            indentation == "spaces" and indent_size or "tab"
        }

        -- Create options table for the third parameter
        local options = {}

        -- Handle additional indent-specific options if they exist in config
        if common.indent.switch_case then
            options.SwitchCase = common.indent.switch_case
        end
        if common.indent.variable_declarator then
            options.VariableDeclarator = common.indent.variable_declarator
        end
        if common.indent.outer_iife_body then
            options.outerIIFEBody = common.indent.outer_iife_body
        end
        if common.indent.member_expression then
            options.MemberExpression = common.indent.member_expression
        end
        if common.indent.function_declaration then
            options.FunctionDeclaration = {
                body = common.indent.function_declaration.body,
                parameters = common.indent.function_declaration.parameters
            }
        end
        if common.indent.function_expression then
            options.FunctionExpression = {
                body = common.indent.function_expression.body,
                parameters = common.indent.function_expression.parameters
            }
        end
        if common.indent.static_block then
            options.StaticBlock = common.indent.static_block
        end
        if common.indent.call_expression then
            options.CallExpression = {
                arguments = common.indent.call_expression.arguments,
            }
        end
        if common.indent.array_expression then
            options.ArrayExpression = common.indent.array_expression
        end
        if common.indent.object_expression then
            options.ObjectExpression = common.indent.object_expression
        end
        if common.indent.import_declaration then
            options.ImportDeclaration = common.indent.import_declaration
        end
        if common.indent.flat_ternary_expressions ~= nil then
            options.flatTernaryExpressions = common.indent.flat_ternary_expressions
        end
        if common.indent.offset_ternary_expressions ~= nil then
            options.offsetTernaryExpressions = common.indent.offset_ternary_expressions
        end
        if common.indent.ignore_nodes then
            if type(common.indent.ignore_nodes) == "string" then
                options.ignoredNodes = { common.indent.ignore_nodes }
            else
                options.ignoredNodes = common.indent.ignore_nodes
            end
        end
        if common.indent.ignore_comments ~= nil then
            options.ignoreComments = common.indent.ignore_comments
        end

        -- Add the options table as third parameter if any options were set
        if next(options) ~= nil then
            table.insert(indentOptions, options)
        end

        -- Special case for tabs with size
        if indentation == "tabs" and indent_size then
            table.insert(indentOptions, 2, indent_size)
        end

        return indentOptions
    end

    local rules = {
        --Indentation rules
        indent = getIndentRule(common.indent),
    }
    -- Prepare the ESLint config as a Lua table
    local eslintConfig = {
        {
            rules = rules,
            plugins = plugins
        }
    }


    -- Convert the table to a JSON string
    return {
        ["eslint.config.js"] = "export default " .. to_json(eslintConfig) .. ";"
    }
end
