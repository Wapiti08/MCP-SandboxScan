# Semantic Cross-Validation

- Dynamic cases: 33
- Cases with called-tool metadata: 33
- Declared egress-risk tools: 19
- Observed network egress: 9
- Declared and observed: 5
- Declared only: 14
- Observed only: 4
- Neither: 10

| Relation | Count |
|----------|-------|
| Declared and observed | 5 |
| Declared only | 14 |
| Observed only | 4 |
| Neither | 10 |

## Cases

| Repo | Called tool | Declared capabilities | Network events | Relation |
|------|-------------|-----------------------|----------------|----------|
| modelcontextprotocol/servers | get-env | unknown | 0 | neither |
| microsoft/playwright-mcp | browser_network_requests | network, browser | 0 | declared_only |
| github/github-mcp-server | get_me | network, code_repo | 18 | declared_and_observed |
| czlonkowski/n8n-mcp | get_node | network, database, credential | 0 | declared_only |
| mksglu/context-mode | ctx_stats | credential | 0 | neither |
| GLips/Figma-Context-MCP | get_figma_data | network, cloud_saas | 0 | declared_only |
| BeehiveInnovations/pal-mcp-server | listmodels | unknown | 2 | observed_only |
| wonderwhy-er/DesktopCommanderMCP | get_file_info | filesystem | 2 | observed_only |
| mobile-next/mobile-mcp | mobile_get_crash | code_repo | 0 | declared_only |
| exa-labs/exa-mcp-server | web_fetch_exa | network, browser | 2 | declared_and_observed |
| makenotion/notion-mcp-server | API-get-self | network, browser, database, credential, cloud_saas | 10 | declared_and_observed |
| Coding-Solo/godot-mcp | get_project_info | filesystem | 0 | neither |
| Pimzino/spec-workflow-mcp | spec-status | filesystem | 0 | neither |
| zcaceres/markdownify-mcp | get-markdown-file | filesystem | 0 | neither |
| GongRzhe/Office-Word-MCP-Server | get_document_info | unknown | 2 | observed_only |
| GongRzhe/Office-PowerPoint-MCP-Server | get_template_file_info | unknown | 0 | neither |
| Flux159/mcp-server-kubernetes | kubectl_context | cloud_saas | 0 | declared_only |
| bitbonsai/mcpvault | get_notes_info | unknown | 0 | neither |
| designcomputer/mysql_mcp_server | get_table_sample | network, browser, database | 0 | declared_only |
| nickclyde/duckduckgo-mcp-server | search | network, browser, database | 2 | declared_and_observed |
| datalayer/jupyter-mcp-server | list_files | filesystem | 0 | neither |
| strowk/mcp-k8s-go | list-k8s-contexts | cloud_saas | 0 | declared_only |
| smn2gnt/MCP-Salesforce | get_record | credential, cloud_saas | 0 | declared_only |
| tsmztech/mcp-server-salesforce | salesforce_read_apex | cloud_saas | 0 | declared_only |
| aashari/mcp-server-atlassian-bitbucket | bb_clone | filesystem, network, code_repo | 0 | declared_only |
| aashari/boilerplate-mcp-server | ip_get_details | network, browser, database, credential | 0 | declared_only |
| aashari/mcp-server-atlassian-jira | jira_get | network, browser, database, credential, code_repo, cloud_saas | 0 | declared_only |
| HatriGt/hana-mcp-server | hana_show_env_vars | unknown | 0 | neither |
| aashari/mcp-server-atlassian-confluence | conf_get | network, browser, database, credential, cloud_saas | 0 | declared_only |
| dend/brick-mcp-app | brick_get_available | unknown | 0 | neither |
| hoangsonww/GitIntel-MCP-Server | hotspots | filesystem, code_repo | 0 | declared_only |
| imprvhub/mcp-claude-hackernews | hn_best | network | 8 | declared_and_observed |
| jhgaylor/hirebase-mcp | get_job | unknown | 2 | observed_only |
