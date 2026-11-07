use std::collections::HashMap;
use std::sync::Arc;

use ryvos_core::error::{Result, RyvosError};
use ryvos_core::traits::Tool;
use ryvos_core::types::{ToolContext, ToolDefinition, ToolIndexEntry, ToolResult};

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: impl Tool) {
        let name = tool.name().to_string();
        self.tools.insert(name, Arc::new(tool));
    }

    pub fn unregister(&mut self, name: &str) -> bool {
        self.tools.remove(name).is_some()
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn list(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|t| ToolDefinition {
                name: t.name().to_string(),
                description: t.description().to_string(),
                input_schema: t.input_schema(),
            })
            .collect()
    }

    pub fn l1_index(&self) -> Vec<ToolIndexEntry> {
        let mut entries: Vec<_> = self
            .tools
            .values()
            .map(|t| ToolIndexEntry {
                name: t.name().to_string(),
                summary: t.description().to_string(),
            })
            .collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries
    }

    pub fn l2_schema(&self, name: &str) -> Option<serde_json::Value> {
        self.tools.get(name).map(|t| {
            serde_json::json!({
                "name": t.name(),
                "description": t.description(),
                "input_schema": t.input_schema(),
            })
        })
    }

    pub fn l2_definition(&self, name: &str) -> Option<ToolDefinition> {
        self.tools.get(name).map(|t| ToolDefinition {
            name: t.name().to_string(),
            description: t.description().to_string(),
            input_schema: t.input_schema(),
        })
    }

    pub async fn execute(
        &self,
        name: &str,
        input: serde_json::Value,
        ctx: ToolContext,
    ) -> Result<ToolResult> {
        let tool = self
            .get(name)
            .ok_or_else(|| RyvosError::ToolNotFound(name.to_string()))?;

        let timeout = std::time::Duration::from_secs(tool.timeout_secs());

        match tokio::time::timeout(timeout, tool.execute(input, ctx)).await {
            Ok(result) => result,
            Err(_) => Err(RyvosError::ToolTimeout {
                tool: name.to_string(),
                timeout_secs: tool.timeout_secs(),
            }),
        }
    }

    pub fn with_builtins() -> Self {
        let mut registry = Self::new();

        registry.register(crate::builtin::bash::BashTool);
        registry.register(crate::builtin::read::ReadTool);
        registry.register(crate::builtin::write::WriteTool);
        registry.register(crate::builtin::edit::EditTool);
        registry.register(crate::builtin::memory_search::MemorySearchTool);
        registry.register(crate::builtin::memory_write::MemoryWriteTool);
        registry.register(crate::builtin::spawn_agent::SpawnAgentTool);
        registry.register(crate::builtin::glob::GlobTool);
        registry.register(crate::builtin::grep::GrepTool);
        registry.register(crate::builtin::web_fetch::WebFetchTool);
        registry.register(crate::builtin::apply_patch::ApplyPatchTool);

        registry.register(crate::builtin::sessions::SessionListTool);
        registry.register(crate::builtin::sessions::SessionHistoryTool);
        registry.register(crate::builtin::sessions::SessionSendTool);
        registry.register(crate::builtin::sessions::SessionSpawnTool);
        registry.register(crate::builtin::sessions::SessionStatusTool);

        registry.register(crate::builtin::memory::MemoryGetTool);
        registry.register(crate::builtin::memory::DailyLogWriteTool);
        registry.register(crate::builtin::memory::MemoryDeleteTool);

        registry.register(crate::builtin::filesystem::FileInfoTool);
        registry.register(crate::builtin::filesystem::FileCopyTool);
        registry.register(crate::builtin::filesystem::FileMoveTool);
        registry.register(crate::builtin::filesystem::FileDeleteTool);
        registry.register(crate::builtin::filesystem::DirListTool);
        registry.register(crate::builtin::filesystem::DirCreateTool);
        registry.register(crate::builtin::filesystem::FileWatchTool);
        registry.register(crate::builtin::filesystem::ArchiveCreateTool);
        registry.register(crate::builtin::filesystem::ArchiveExtractTool);

        registry.register(crate::builtin::git::GitStatusTool);
        registry.register(crate::builtin::git::GitDiffTool);
        registry.register(crate::builtin::git::GitLogTool);
        registry.register(crate::builtin::git::GitCommitTool);
        registry.register(crate::builtin::git::GitBranchTool);
        registry.register(crate::builtin::git::GitCloneTool);

        registry.register(crate::builtin::code::CodeFormatTool);
        registry.register(crate::builtin::code::CodeLintTool);
        registry.register(crate::builtin::code::TestRunTool);
        registry.register(crate::builtin::code::CodeOutlineTool);

        registry.register(crate::builtin::network::HttpRequestTool);
        registry.register(crate::builtin::network::HttpDownloadTool);
        registry.register(crate::builtin::network::DnsLookupTool);
        registry.register(crate::builtin::network::NetworkCheckTool);

        registry.register(crate::builtin::system::ProcessListTool);
        registry.register(crate::builtin::system::ProcessKillTool);
        registry.register(crate::builtin::system::EnvGetTool);
        registry.register(crate::builtin::system::SystemInfoTool);
        registry.register(crate::builtin::system::DiskUsageTool);

        registry.register(crate::builtin::data::JsonQueryTool);
        registry.register(crate::builtin::data::CsvParseTool);
        registry.register(crate::builtin::data::YamlConvertTool);
        registry.register(crate::builtin::data::TomlConvertTool);
        registry.register(crate::builtin::data::Base64CodecTool);
        registry.register(crate::builtin::data::HashComputeTool);
        registry.register(crate::builtin::data::RegexReplaceTool);
        registry.register(crate::builtin::data::TextDiffTool);

        registry.register(crate::builtin::scheduling::CronListTool);
        registry.register(crate::builtin::scheduling::CronAddTool);
        registry.register(crate::builtin::scheduling::CronRemoveTool);

        registry.register(crate::builtin::database::SqliteQueryTool);
        registry.register(crate::builtin::database::SqliteSchemaTool);

        registry.register(crate::builtin::notification::NotificationSendTool);

        crate::builtin::browser::register_browser_tools(&mut registry);

        registry.register(crate::builtin::viking::VikingSearchTool);
        registry.register(crate::builtin::viking::VikingReadTool);
        registry.register(crate::builtin::viking::VikingWriteTool);
        registry.register(crate::builtin::viking::VikingListTool);

        registry
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ryvos_test_utils::{test_tool_context, MockTool};

    #[test]
    fn registry_register_and_get() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("my_tool"));
        assert!(registry.get("my_tool").is_some());
        assert!(registry.get("other").is_none());
    }

    #[test]
    fn registry_unregister() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("removable"));
        assert!(registry.unregister("removable"));
        assert!(registry.get("removable").is_none());
        assert!(!registry.unregister("removable"));
    }

    #[test]
    fn registry_list_returns_all_names() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("alpha"));
        registry.register(MockTool::new("beta"));
        registry.register(MockTool::new("gamma"));
        let mut names = registry.list();
        names.sort();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn registry_definitions_returns_all() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("tool_a").with_description("desc A"));
        registry.register(MockTool::new("tool_b").with_description("desc B"));
        let defs = registry.definitions();
        assert_eq!(defs.len(), 2);
        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"tool_a"));
        assert!(names.contains(&"tool_b"));
    }

    #[tokio::test]
    async fn registry_execute_calls_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("exec_tool"));
        let ctx = test_tool_context();
        let result = registry
            .execute("exec_tool", serde_json::json!({"x": 1}), ctx)
            .await
            .unwrap();
        assert!(!result.is_error);
        assert_eq!(result.content, "mock output");
    }

    #[tokio::test]
    async fn registry_execute_unknown_tool_errors() {
        let registry = ToolRegistry::new();
        let ctx = test_tool_context();
        let result = registry
            .execute("no_such_tool", serde_json::json!({}), ctx)
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn registry_with_builtins_has_tools() {
        let registry = ToolRegistry::with_builtins();
        let names = registry.list();
        assert!(names.contains(&"read"));
        assert!(names.contains(&"write"));
        assert!(names.contains(&"edit"));
        assert!(names.contains(&"glob"));
        assert!(names.contains(&"json_query"));
        assert!(names.contains(&"csv_parse"));
        assert!(names.contains(&"file_info"));
        assert!(names.contains(&"bash"));
        assert!(names.len() > 40);
    }

    #[test]
    fn registry_l1_index_and_l2_schema() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("alpha").with_description("Alpha desc"));
        registry.register(MockTool::new("beta").with_description("Beta desc"));

        let index = registry.l1_index();
        assert_eq!(index.len(), 2);
        assert_eq!(index[0].name, "alpha");
        assert_eq!(index[0].summary, "Alpha desc");
        assert_eq!(index[1].name, "beta");
        assert_eq!(index[1].summary, "Beta desc");

        let schema = registry.l2_schema("alpha");
        assert!(schema.is_some());
        let val = schema.unwrap();
        assert_eq!(val["name"], "alpha");
        assert_eq!(val["description"], "Alpha desc");

        assert!(registry.l2_schema("non_existent").is_none());
        assert!(registry.l2_definition("alpha").is_some());
        assert!(registry.l2_definition("non_existent").is_none());
    }

    #[test]
    fn registry_register_overwrites_same_name() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("dup").with_description("first"));
        registry.register(MockTool::new("dup").with_description("second"));
        let defs = registry.definitions();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].description, "second");
    }
}
