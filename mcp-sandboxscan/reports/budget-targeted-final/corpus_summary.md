# Corpus Scan: run-1788723072

- Repos: 35
- Resolved: 35
- Scanned: 27
- Scan success: 77.1%
- Suspicious rate: 37.0% (flows observed, NOT verified vulns)

## Tier-1 (dedicated MCP servers)

- Resolved: 28
- Scanned: 21
- Scan success: 75.0%
- Suspicious rate: 38.1%

- Total latency p50/p95: 5323ms / 32547ms

## Latency (successful scans)

- Count: 27
- Build p50/p95: 4041ms / 31314ms
- Scan p50/p95: 308ms / 4394ms
- Total p50/p95: 5323ms / 53628ms

## Tool metadata and semantic profile

| Subset | Repos with metadata | Tools | Described | Sensitive |
|--------|---------------------|-------|-----------|-----------|
| All | 27 | 442 | 442 | 331 |
| Tier-1 | 21 | 368 | 368 | 288 |
| Tier-2 | 6 | 74 | 74 | 43 |

### Semantic capabilities

| Capability | Tools |
|------------|-------|
| browser | 176 |
| unknown | 111 |
| network | 109 |
| cloud_saas | 97 |
| database | 86 |
| code_repo | 77 |
| filesystem | 66 |
| credential | 31 |
| shell | 27 |

## By WASM class

| Class | Total | Scanned | Suspicious |
|-------|-------|---------|------------|
| unknown | 1 | 1 | 0 |
| wasm-hard | 24 | 23 | 8 |
| wasm-needs-runtime | 8 | 1 | 0 |
| wasm-ready | 2 | 2 | 2 |

## By ecosystem

| Ecosystem | Total | Scanned | Rate |
|-----------|-------|---------|------|
| go | 2 | 2 | 100.0% |
| npm | 25 | 24 | 96.0% |
| python | 8 | 1 | 12.5% |

## Failure categories

| Category | Count |
|----------|-------|
| mcp_start | 7 |
| timeout | 1 |

## Cases

| Repo | Tier | OK | Flows | Deps | Stars | Total ms | Error |
|------|------|----|-------|------|-------|----------|-------|
| modelcontextprotocol/servers | tier1 | true | 1 | 4 | 87415 | 7990 |  |
| microsoft/playwright-mcp | tier1 | true | 0 | 5 | 34075 | 648 |  |
| github/github-mcp-server | tier1 | true | 13 | 2 | 30784 | 30557 |  |
| czlonkowski/n8n-mcp | tier1 | true | 2 | 40 | 21838 | 32547 |  |
| mksglu/context-mode | tier2 | true | 0 | 15 | 17727 | 5413 |  |
| GLips/Figma-Context-MCP | tier1 | true | 0 | 31 | 15148 | 6362 |  |
| BeehiveInnovations/pal-mcp-server | tier1 | false | 0 | 6 | 11597 | 8487 | failed to scan native MCP subject BeehiveInnovations__pal-mcp-server: MCP server closed stdout before JSON response |
| wonderwhy-er/DesktopCommanderMCP | tier1 | true | 4 | 47 | 6184 | 14902 |  |
| executeautomation/mcp-playwright | tier1 | true | 7 | 19 | 5554 | 8228 |  |
| mobile-next/mobile-mcp | tier1 | true | 8 | 26 | 5228 | 2658 |  |
| 21st-dev/magic-mcp | tier1 | false | 0 | 15 | 5166 | 275317 | failed to scan native MCP subject 21st-dev__magic-mcp: MCP server timed out after 90s |
| exa-labs/exa-mcp-server | tier1 | true | 0 | 17 | 4590 | 896 |  |
| makenotion/notion-mcp-server | tier1 | true | 0 | 24 | 4437 | 1242 |  |
| Coding-Solo/godot-mcp | tier1 | true | 0 | 5 | 4259 | 3095 |  |
| Pimzino/spec-workflow-mcp | tier1 | true | 1 | 52 | 4234 | 20857 |  |
| zcaceres/markdownify-mcp | tier2 | true | 2 | 6 | 2749 | 8474 |  |
| GongRzhe/Office-Word-MCP-Server | tier1 | true | 0 | 5 | 2057 | 6863 |  |
| GongRzhe/Office-PowerPoint-MCP-Server | tier1 | false | 0 | 4 | 1798 | 5182 | failed to scan native MCP subject GongRzhe__Office-PowerPoint-MCP-Server: MCP server closed stdout before JSON response |
| Flux159/mcp-server-kubernetes | tier1 | true | 0 | 19 | 1438 | 5323 |  |
| bitbonsai/mcpvault | tier2 | true | 0 | 8 | 1432 | 2115 |  |
| designcomputer/mysql_mcp_server | tier1 | false | 0 | 4 | 1304 | 4742 | failed to scan native MCP subject designcomputer__mysql_mcp_server: MCP server closed stdout before JSON response |
| nickclyde/duckduckgo-mcp-server | tier1 | false | 0 | 1 | 1254 | 4990 | failed to scan native MCP subject nickclyde__duckduckgo-mcp-server: MCP server closed stdout before JSON response |
| datalayer/jupyter-mcp-server | tier1 | false | 0 | 7 | 1167 | 6539 | failed to scan native MCP subject datalayer__jupyter-mcp-server: MCP server closed stdout before JSON response |
| strowk/mcp-k8s-go | tier1 | true | 8 | 2 | 382 | 53628 |  |
| smn2gnt/MCP-Salesforce | tier1 | false | 0 | 4 | 179 | 5301 | failed to scan native MCP subject smn2gnt__MCP-Salesforce: MCP server closed stdout before JSON response |
| tsmztech/mcp-server-salesforce | tier1 | true | 0 | 6 | 160 | 3117 |  |
| aashari/mcp-server-atlassian-bitbucket | tier1 | true | 0 | 34 | 156 | 5469 |  |
| aashari/boilerplate-mcp-server | tier1 | true | 0 | 34 | 71 | 4647 |  |
| aashari/mcp-server-atlassian-jira | tier1 | true | 0 | 37 | 71 | 4357 |  |
| HatriGt/hana-mcp-server | tier1 | true | 0 | 4 | 58 | 383 |  |
| aashari/mcp-server-atlassian-confluence | tier1 | true | 0 | 36 | 58 | 3904 |  |
| dend/brick-mcp-app | tier2 | true | 2 | 22 | 16 | 186785 |  |
| hoangsonww/GitIntel-MCP-Server | tier2 | true | 0 | 7 | 11 | 2058 |  |
| imprvhub/mcp-claude-hackernews | tier2 | true | 0 | 4 | 10 | 1572 |  |
| jhgaylor/hirebase-mcp | tier2 | false | 0 | 3 | 10 | 5075 | failed to scan native MCP subject jhgaylor__hirebase-mcp: MCP server closed stdout before JSON response |
