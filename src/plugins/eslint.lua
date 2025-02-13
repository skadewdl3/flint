function Details()
    local details = {
        id = "eslint",
        author = "Soham Karandikar",
        version = "0.0.1",
        extensions = { "js", "ts", "mjs", "cjs" },
        category = "linter"
    }
    return details
end

function Validate(config)
    -- sleep(2000)
    temp = { foo = 'bar', baz = 123, qux = { nested = true } }
    log.debug(temp);
    return true
end

function Generate(config)
    local common = config.common
    log.info("Generating ESLint configuration")
    log.debug(common)

    local function getIndentRule(indentation, indent_size)
        -- Base indentation configuration
        local indentOptions = {
            "error",
            indentation == "spaces" and indent_size or "tab"
        }

        -- Create options table for the third parameter
        local options = {}

        -- Handle additional indent-specific options if they exist in config
        if common.switch_case then
            options.SwitchCase = common.switch_case
        end
        if common.variable_declarator then
            options.VariableDeclarator = common.variable_declarator
        end
        if common.outer_iife_body then
            options.outerIIFEBody = common.outer_iife_body
        end
        if common.member_expression then
            options.MemberExpression = common.member_expression
        end
        if common.function_declaration then
            options.FunctionDeclaration = {
                body = common.function_declaration.body,
                parameters = common.function_declaration.parameters
            }
        end
        if common.function_expression then
            options.FunctionExpression = {
                body = common.function_expression.body,
                parameters = common.function_expression.parameters
            }
        end
        if common.static_block then
            options.StaticBlock = common.static_block
        end
        if common.call_expression then
            options.CallExpression = {
                arguments = common.call_expression.arguments,
            }
        end
        if common.array_expression then
            options.ArrayExpression = common.array_expression
        end
        if common.object_expression then
            options.ObjectExpression = common.object_expression
        end
        if common.import_declaration then
            options.ImportDeclaration = common.import_declaration
        end
        if common.flat_ternary_expressions ~= nil then
            options.flatTernaryExpressions = common.flat_ternary_expressions
        end
        if common.offset_ternary_expressions ~= nil then
            options.offsetTernaryExpressions = common.offset_ternary_expressions
        end
        if common.ignore_nodes then
            if type(common.ignore_nodes) == "string" then
                options.ignoredNodes = { common.ignore_nodes }
            else
                options.ignoredNodes = common.ignore_nodes
            end
        end
        if common.ignore_comments ~= nil then
            options.ignoreComments = common.ignore_comments
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
        indent = getIndentRule(common.indentation, common.indent_size),
    }
    -- Prepare the ESLint config as a Lua table
    local eslintConfig = {
        { 
            rules = rules,
        }
    }

    -- Convert the table to a JSON string
    return {
        ["eslint.config.js"] = "export default " .. to_json(eslintConfig) .. ";"
    }
end
