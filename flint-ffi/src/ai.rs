use async_openai::config::Config;
use async_openai::{Client, config::OpenAIConfig};
use flint_utils::debug;
use mlua::{Lua, Result as LuaResult, Table, UserData, Variadic};
use serde::{Deserialize, Serialize};
use serde_json::json;

struct ClientWrapper<C: Config> {
    pub client: Client<C>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiResponse {
    pub suggestion: String,
    pub explanation: String,
    pub code_language: String,
    pub code_changes: String,
    pub line_no: u32,
    pub file: String,
}

impl UserData for AiResponse {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("suggestion", |_, this: &Self| Ok(this.suggestion.clone()));
        fields.add_field_method_get("explanation", |_, this: &Self| Ok(this.explanation.clone()));
        fields.add_field_method_get("code_changes", |_, this: &Self| {
            Ok(this.code_changes.clone())
        });
        fields.add_field_method_get("code_language", |_, this: &Self| {
            Ok(this.code_language.clone())
        });
        fields.add_field_method_get("file", |_, this: &Self| Ok(this.file.clone()));
        fields.add_field_method_get("line_no", |_, this: &Self| Ok(this.line_no));
    }
}

impl UserData for ClientWrapper<OpenAIConfig> {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("test", |_, this, _: ()| async move {
            let response: serde_json::Value = this
                .client
                .chat()
                .create_byot(serde_json::json! {
                    {
                        "messages": [
                            {
                                "role": "system",
                                "content": "You are a helpful assistant."
                            },
                            {
                                "role": "user",
                                "content": "How does large language model work?"
                            }
                        ],
                        "model": this.model
                    }
                })
                .await
                .map_err(|e| mlua::Error::runtime(format!("{:#?}", e)))?;

            debug!("{:#?}", response);

            Ok(())
        });

        methods.add_async_method("prompt", |lua, this, msgs: Variadic<Table>| async move {
            let mut messages = Vec::new();
            messages.push({
                let tbl = lua.create_table()?;
                tbl.set("role", "system")?;
                tbl.set("content", r#"You are a helpful coding assistant.
                    You will be provided with code snippets and you will provide a response with logical, architectural
                    and bottleneck improvements to the code snippet. You will provide explanation with changed code only.
                    If you don't understand the code, or lack context to make a useful suggestion, you will not provide
                    any suggestions and will specify that you lack the required information. You will limit your response
                    to 300 characters for the explanation, and only the required code changes. Output a JSON array.
                    Every element in the JSON array should have the following fields:

                    - suggestion: A one line suggestion telling what changes are needed. This should be a single line only.
                    - explanation: An explanation as to why the suggestion is made. This should be not more than a 100 words.
                    - code_language: The programming language of the code changes. This should be comptaible with markdown languages.
                    For example, "Rust" won't work since ```Rust doesn't work in markdown. It has to be "rust", since ```rust works in markdown.
                    - code_changes: The code changes needed to implement the suggestion.
                    - file: The path of the file relative to the project directory.
                    - line_no: The line number where this suggestion should be applied.


                    Here's an example of the output:
                    [
                    {
                    "suggestion": "Suggestion 1",
                    "explanation": "Explanation for suggestion 1",
                    "code_language": "rust",
                    "code_changes": "Code changes for suggestion 1",
                    "line_no": 10,
                    "file": "Name of the file"
                    },
                    {
                    "suggestion": "Suggestion 2",
                    "explanation": "Explanation for suggestion 2",
                    "code_language": "js",
                    "code_changes": "Code changes for suggestion 2",
                    "line_no": 1,
                    "file": "Name of the file"
                    }
                    ]"#

                )?;
                tbl
            });
            for msg in msgs {
                messages.push(msg);
            }


            let response: serde_json::Value = this
                .client
                .chat()
                .create_byot(json! {
                    {
                        "messages":  messages,
                        "model": this.model,
                        "response_format": {
                            "type": "json_schema",
                            "json_schema": {
                                "name": "code_improvement",
                                "schema": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "suggestion": { "type": "string" },
                                            "explanation": { "type": "string" },
                                            "code_changes": { "type": "string" },
                                            "code_language": { "type": "string" },
                                            "file": { "type": "string" },
                                            "line_no": { "type": "number" },
                                        },
                                        "required": ["suggestion", "explanation", "code_changes", "code_language"],
                                        "additionalProperties": false
                                    }
                                },
                                "strict": true
                            }
                        }
                    }
                })
                .await
                .map_err(|e| mlua::Error::runtime(format!("{:#?}", e)))?;


            debug!("{:#?}", response);
            let response = response["choices"][0]["message"]["content"].clone();

            // Parse once to handle escaped characters
            let response_str: String = serde_json::from_value(response)
                .map_err(|e| mlua::Error::runtime(format!("{:#?}", e)))?;

            // Parse again to deserialize the JSON string into a Vec<AiResponse>
            let response: Vec<AiResponse> = serde_json::from_str(&response_str)
                .map_err(|e| mlua::Error::runtime(format!("{:#?}", e)))?;

            // debug!("{:#?}", response);

            let table = lua.create_table()?;
            for (i, response) in response.iter().enumerate() {
                let tbl = lua.create_table()?;
                tbl.set("suggestion", response.suggestion.clone())?;
                tbl.set("explanation", response.explanation.clone())?;
                tbl.set("code_changes", response.code_changes.clone())?;
                tbl.set("code_language", response.code_language.clone())?;
                tbl.set("file", response.file.clone())?;
                tbl.set("line_no", response.line_no.clone())?;
                table.set(i + 1, tbl)?;
            }

            Ok(table)
        });
    }
}

pub fn ai_helpers(lua: &Lua) -> LuaResult<Table> {
    let table = lua.create_table()?;

    let create_client = lua.create_function(|_, args: Table| {
        let api_key: String = args.get("api_key")?;
        let api_base_url: String = args.get("api_base_url")?;
        let model: String = args.get("model")?;
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base_url);
        let client = Client::with_config(config);
        Ok(ClientWrapper { client, model })
    })?;
    table.set("create_client", create_client)?;

    let message = lua.create_table()?;
    let message_assistant = lua.create_function(|lua, msg: String| {
        let res = lua.create_table()?;
        res.set("role", "assistant")?;
        res.set("content", msg)?;
        Ok(res)
    })?;

    let message_user = lua.create_function(|lua, msg: String| {
        let res = lua.create_table()?;
        res.set("role", "user")?;
        res.set("content", msg)?;
        Ok(res)
    })?;

    let message_system = lua.create_function(|lua, msg: String| {
        let res = lua.create_table()?;
        res.set("role", "system")?;
        res.set("content", msg)?;
        Ok(res)
    })?;
    message.set("user", message_user)?;
    message.set("system", message_system)?;
    message.set("assistant", message_assistant)?;

    table.set("message", message)?;

    Ok(table)
}
