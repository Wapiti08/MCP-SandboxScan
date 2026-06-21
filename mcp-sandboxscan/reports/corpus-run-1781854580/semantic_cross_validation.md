# Semantic Cross-Validation

- Dynamic cases: 35
- Cases with called-tool metadata: 35
- Declared egress-risk tools: 22
- Observed network egress: 7
- Declared and observed: 5
- Declared only: 17
- Observed only: 2
- Neither: 11

| Relation | Count |
|----------|-------|
| Declared and observed | 5 |
| Declared only | 17 |
| Observed only | 2 |
| Neither | 11 |

## Cases

| Repo | Called tool | Declared capabilities | Network events | Relation |
|------|-------------|-----------------------|----------------|----------|
| modelcontextprotocol/servers | echo | unknown | 0 | neither |
| microsoft/playwright-mcp | browser_close | browser | 0 | declared_only |
| github/github-mcp-server | add_comment_to_pending_review | network, code_repo | 4 | declared_and_observed |
| czlonkowski/n8n-mcp | tools_documentation | unknown | 0 | neither |
| mksglu/context-mode | ctx_execute | shell, network, database, code_repo | 0 | declared_only |
| GLips/Figma-Context-MCP | get_figma_data | network, cloud_saas | 0 | declared_only |
| BeehiveInnovations/pal-mcp-server | chat | filesystem | 0 | neither |
| wonderwhy-er/DesktopCommanderMCP | get_config | shell | 2 | observed_only |
| executeautomation/mcp-playwright | start_codegen_session | filesystem, browser | 0 | declared_only |
| mobile-next/mobile-mcp | mobile_list_available_devices | unknown | 0 | neither |
| 21st-dev/magic-mcp | 21st_magic_component_builder | filesystem, network, browser, database | 0 | declared_only |
| exa-labs/exa-mcp-server | web_search_exa | network, browser, database | 2 | declared_and_observed |
| makenotion/notion-mcp-server | API-get-user | network, browser, database, cloud_saas | 2 | declared_and_observed |
| Coding-Solo/godot-mcp | launch_editor | filesystem | 0 | neither |
| Pimzino/spec-workflow-mcp | spec-workflow-guide | network | 0 | declared_only |
| zcaceres/markdownify-mcp | audio-to-markdown | filesystem | 0 | neither |
| GongRzhe/Office-Word-MCP-Server | create_document | unknown | 2 | observed_only |
| GongRzhe/Office-PowerPoint-MCP-Server | create_presentation | unknown | 0 | neither |
| Flux159/mcp-server-kubernetes | cleanup | unknown | 0 | neither |
| bitbonsai/mcpvault | read_note | unknown | 0 | neither |
| designcomputer/mysql_mcp_server | execute_sql | browser, database | 0 | declared_only |
| nickclyde/duckduckgo-mcp-server | search | network, browser, database | 0 | declared_only |
| datalayer/jupyter-mcp-server | list_files | filesystem | 0 | neither |
| strowk/mcp-k8s-go | apply-k8s-resource | cloud_saas | 0 | declared_only |
| smn2gnt/MCP-Salesforce | run_soql_query | database, credential, cloud_saas | 0 | declared_only |
| tsmztech/mcp-server-salesforce | salesforce_search_objects | cloud_saas | 0 | declared_only |
| aashari/mcp-server-atlassian-bitbucket | bb_get | network, browser, database, credential, code_repo, cloud_saas | 0 | declared_only |
| aashari/boilerplate-mcp-server | ip_get_details | network, browser, database, credential | 0 | declared_only |
| aashari/mcp-server-atlassian-jira | jira_get | network, browser, database, credential, code_repo, cloud_saas | 0 | declared_only |
| HatriGt/hana-mcp-server | hana_show_config | browser, database | 0 | declared_only |
| aashari/mcp-server-atlassian-confluence | conf_get | network, browser, database, credential, cloud_saas | 0 | declared_only |
| dend/brick-mcp-app | brick_read_me | unknown | 0 | neither |
| hoangsonww/GitIntel-MCP-Server | hotspots | filesystem, code_repo | 0 | declared_only |
| imprvhub/mcp-claude-hackernews | hn_latest | network | 2 | declared_and_observed |
| jhgaylor/hirebase-mcp | search_jobs | database | 2 | declared_and_observed |
