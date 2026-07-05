//! Native workspace file tools backed by [`nest_file::FileService`].

use async_trait::async_trait;
use nest_file::{search_files, FileSearchOptions, FileService};
use nest_mcp::split_qualified_tool_name;
use serde_json::{json, Value};
use tokio::task;

use crate::tool::AgentTool;
use crate::{NestError, NestResult};
use crate::tools::ToolSource;

/// Virtual server id for in-process file tools.
pub const FILE_SERVER: &str = "nest-file";

/// Executes scoped file operations for the agent loop.
#[derive(Clone)]
pub struct FileToolSource {
    files: FileService,
}

impl FileToolSource {
    /// Creates a file tool source over the given service.
    pub fn new(files: FileService) -> Self {
        Self { files }
    }

    /// Tool metadata exposed to the model.
    pub fn tool_definitions() -> Vec<AgentTool> {
        vec![
            tool("read_file", "Read a UTF-8 text file from the project workspace.", json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path relative to the project root" }
                },
                "required": ["path"]
            })),
            tool("write_file", "Write UTF-8 text to a file (creates or overwrites).", json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"]
            })),
            tool("update_file", "Replace text in an existing file. Call read_file first and copy \
                 old_string exactly (including whitespace). Set replace_all to true for every match. \
                 Empty old_string prepends new_string to the file. Empty new_string deletes the first \
                 match (or all matches when replace_all is true).", json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path relative to the project root" },
                    "old_string": {
                        "type": "string",
                        "description": "Exact text to replace; empty string prepends new_string"
                    },
                    "new_string": {
                        "type": "string",
                        "description": "Replacement text; empty string deletes old_string"
                    },
                    "replace_all": { "type": "boolean" }
                },
                "required": ["path", "old_string", "new_string"]
            })),
            tool("delete_path", "Delete a file or directory.", json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "recursive": { "type": "boolean" }
                },
                "required": ["path"]
            })),
            tool("create_directory", "Create a directory and any missing parents.", json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            })),
            tool("list_directory", "List entries in a directory as JSON.", json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                }
            })),
            tool("search_files", "Search for files and directories by path substring. Each word in \
                 query must appear in the relative path (case-insensitive). Skips build and VCS \
                 directories. Use before read_file when you do not know the exact path.", json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Path search terms, e.g. \"agent mod.rs\" or \"sidebar search\""
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory to search from (default: project root)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum matches to return (default 50, max 500)"
                    }
                },
                "required": ["query"]
            })),
        ]
    }
}

fn tool(name: &str, description: &str, input_schema: Value) -> AgentTool {
    AgentTool::native(FILE_SERVER, name, description, input_schema)
}

#[async_trait]
impl ToolSource for FileToolSource {
    async fn list_tools(&mut self) -> NestResult<Vec<AgentTool>> {
        Ok(Self::tool_definitions())
    }

    async fn call_tool(&mut self, qualified_name: &str, arguments: Value) -> NestResult<String> {
        let (server, tool) = split_qualified_tool_name(qualified_name)?;
        if server != FILE_SERVER {
            return Err(NestError::network(format!(
                "file tool source cannot call {qualified_name}"
            ))
            .with_module("nest-agent"));
        }

        let files = self.files.clone();
        let tool = tool.to_string();
        task::spawn_blocking(move || execute_file_tool(&files, &tool, &arguments))
            .await
            .map_err(|error| NestError::network(error.to_string()).with_module("nest-agent"))?
    }
}

fn execute_file_tool(files: &FileService, tool: &str, arguments: &Value) -> NestResult<String> {
    match tool {
        "read_file" => {
            let path = required_str(arguments, "path")?;
            Ok(files.read_text(path)?)
        }
        "write_file" => {
            let path = required_str(arguments, "path")?;
            let content = required_str(arguments, "content")?;
            files.write_text(path, content)?;
            Ok(format!("Wrote {} bytes to {path}.", content.len()))
        }
        "update_file" => {
            let path = required_str(arguments, "path")?;
            let old = optional_str(arguments, "old_string");
            let new = optional_str(arguments, "new_string");
            let replace_all = arguments
                .get("replace_all")
                .and_then(Value::as_bool)
                .unwrap_or(false);

            if old.is_empty() {
                if new.is_empty() {
                    return Err(NestError::validation(
                        "update_file requires non-empty new_string when old_string is empty",
                    ));
                }
                let mut content = files.read_text(path)?;
                content = prepend_text(content, new);
                files.write_text(path, &content)?;
                return Ok(format!("Prepended to {path}."));
            }

            let mut content = files.read_text(path)?;
            let count = if new.is_empty() {
                delete_matches(&mut content, old, replace_all, path)?
            } else if replace_all {
                replace_all_matches(&mut content, old, new, path)?
            } else {
                replace_first_match(&mut content, old, new, path)?
            };
            files.write_text(path, &content)?;
            Ok(format!("Updated {count} occurrence(s) in {path}."))
        }
        "delete_path" => {
            let path = required_str(arguments, "path")?;
            let recursive = arguments
                .get("recursive")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let metadata = files.metadata(path)?;
            if metadata.is_dir {
                files.delete_dir(path, recursive)?;
                Ok(format!("Deleted directory {path}."))
            } else if metadata.is_file {
                if recursive {
                    return Err(NestError::validation(format!(
                        "{path} is a file; omit recursive to delete files"
                    )));
                }
                files.delete_file(path)?;
                Ok(format!("Deleted file {path}."))
            } else {
                Err(NestError::validation(format!(
                    "{path} is not a regular file or directory"
                )))
            }
        }
        "create_directory" => {
            let path = required_str(arguments, "path")?;
            files.create_dir_all(path)?;
            Ok(format!("Created directory {path}."))
        }
        "list_directory" => {
            let path = arguments
                .get("path")
                .and_then(Value::as_str)
                .unwrap_or(".");
            let entries = files.list_dir(path)?;
            let payload: Vec<Value> = entries
                .into_iter()
                .map(|entry| {
                    json!({
                        "name": entry.name,
                        "path": entry.path.display().to_string(),
                        "is_dir": entry.metadata.is_dir,
                        "size": entry.metadata.len,
                    })
                })
                .collect();
            serde_json::to_string_pretty(&payload).map_err(|error| {
                NestError::validation(format!("failed to encode directory listing: {error}"))
            })
        }
        "search_files" => {
            let query = required_str(arguments, "query")?;
            let path = arguments
                .get("path")
                .and_then(Value::as_str)
                .unwrap_or(".");
            let max_results = arguments
                .get("max_results")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(50);
            let matches = search_files(
                files,
                &FileSearchOptions::for_query(query).with_scope(path, max_results),
            )?;
            let payload: Vec<Value> = matches
                .into_iter()
                .map(|entry| {
                    json!({
                        "path": entry.path,
                        "is_dir": entry.is_dir,
                    })
                })
                .collect();
            serde_json::to_string_pretty(&payload).map_err(|error| {
                NestError::validation(format!("failed to encode search results: {error}"))
            })
        }
        other => Err(NestError::network(format!("unknown file tool: {other}"))
            .with_module("nest-agent")),
    }
}

fn required_str<'a>(arguments: &'a Value, field: &str) -> NestResult<&'a str> {
    arguments
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| NestError::validation(format!("missing or empty `{field}`")))
}

fn optional_str<'a>(arguments: &'a Value, field: &str) -> &'a str {
    arguments
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn prepend_text(content: String, new: &str) -> String {
    if content.is_empty() {
        new.to_string()
    } else if new.ends_with('\n') || content.starts_with('\n') {
        format!("{new}{content}")
    } else {
        format!("{new}\n{content}")
    }
}

fn replace_first_match(
    content: &mut String,
    old: &str,
    new: &str,
    path: &str,
) -> NestResult<usize> {
    if !content.contains(old) {
        return Err(not_found_error(path));
    }
    *content = content.replacen(old, new, 1);
    Ok(1)
}

fn replace_all_matches(
    content: &mut String,
    old: &str,
    new: &str,
    path: &str,
) -> NestResult<usize> {
    let matches = content.matches(old).count();
    if matches == 0 {
        return Err(not_found_error(path));
    }
    *content = content.replace(old, new);
    Ok(matches)
}

fn delete_matches(
    content: &mut String,
    old: &str,
    replace_all: bool,
    path: &str,
) -> NestResult<usize> {
    if replace_all {
        replace_all_matches(content, old, "", path)
    } else {
        replace_first_match(content, old, "", path)
    }
}

fn not_found_error(path: &str) -> NestError {
    NestError::validation(format!(
        "text to replace was not found in {path}; call read_file and copy the exact snippet \
         including whitespace"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_file::FileModule;
    use nest_core::AppBuilder;
    use tempfile::tempdir;

    fn scoped_files(root: &std::path::Path) -> FileService {
        AppBuilder::new()
            .module(FileModule::scoped(root))
            .build()
            .unwrap()
            .context
            .service::<FileService>()
            .unwrap()
            .clone()
    }

    #[tokio::test]
    async fn write_and_read_via_tool_source() {
        let dir = tempdir().unwrap();
        let mut source = FileToolSource::new(scoped_files(dir.path()));

        let write_result = source
            .call_tool(
                "nest-file/write_file",
                json!({"path": "note.txt", "content": "hello"}),
            )
            .await
            .unwrap();
        assert!(write_result.contains("note.txt"));

        let content = source
            .call_tool("nest-file/read_file", json!({"path": "note.txt"}))
            .await
            .unwrap();
        assert_eq!(content, "hello");
    }

    #[tokio::test]
    async fn update_file_prepends_when_old_string_empty() {
        let dir = tempdir().unwrap();
        let mut source = FileToolSource::new(scoped_files(dir.path()));

        source
            .call_tool(
                "nest-file/write_file",
                json!({"path": "mod.rs", "content": "fn main() {}"}),
            )
            .await
            .unwrap();

        source
            .call_tool(
                "nest-file/update_file",
                json!({
                    "path": "mod.rs",
                    "old_string": "",
                    "new_string": "//! Module docs.\n"
                }),
            )
            .await
            .unwrap();

        let content = source
            .call_tool("nest-file/read_file", json!({"path": "mod.rs"}))
            .await
            .unwrap();
        assert_eq!(content, "//! Module docs.\nfn main() {}");
    }

    #[tokio::test]
    async fn update_file_deletes_when_new_string_empty() {
        let dir = tempdir().unwrap();
        let mut source = FileToolSource::new(scoped_files(dir.path()));

        source
            .call_tool(
                "nest-file/write_file",
                json!({"path": "mod.rs", "content": "keep\nremove\nkeep"}),
            )
            .await
            .unwrap();

        source
            .call_tool(
                "nest-file/update_file",
                json!({
                    "path": "mod.rs",
                    "old_string": "remove\n",
                    "new_string": ""
                }),
            )
            .await
            .unwrap();

        let content = source
            .call_tool("nest-file/read_file", json!({"path": "mod.rs"}))
            .await
            .unwrap();
        assert_eq!(content, "keep\nkeep");
    }

    #[tokio::test]
    async fn search_files_via_tool_source() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src/agent")).unwrap();
        std::fs::write(dir.path().join("src/agent/mod.rs"), "mod").unwrap();
        let mut source = FileToolSource::new(scoped_files(dir.path()));

        let payload = source
            .call_tool(
                "nest-file/search_files",
                json!({"query": "agent mod.rs", "max_results": 10}),
            )
            .await
            .unwrap();
        assert!(payload.contains("src/agent/mod.rs"));
    }
}
