# Corpus Scan: run-1788722305

- Repos: 35
- Resolved: 35
- Scanned: 28
- Scan success: 80.0%
- Suspicious rate: 0.0% (flows observed, NOT verified vulns)

## Tier-1 (dedicated MCP servers)

- Resolved: 28
- Scanned: 22
- Scan success: 78.6%
- Suspicious rate: 0.0%

- Total latency p50/p95: 10651ms / 91836ms

## Latency (successful scans)

- Count: 28
- Build p50/p95: 9300ms / 91400ms
- Scan p50/p95: 173ms / 1219ms
- Total p50/p95: 9460ms / 100981ms

## Tool metadata and semantic profile

| Subset | Repos with metadata | Tools | Described | Sensitive |
|--------|---------------------|-------|-----------|-----------|
| All | 28 | 446 | 446 | 335 |
| Tier-1 | 22 | 372 | 372 | 292 |
| Tier-2 | 6 | 74 | 74 | 43 |

### Semantic capabilities

| Capability | Tools |
|------------|-------|
| browser | 178 |
| network | 113 |
| unknown | 111 |
| cloud_saas | 98 |
| database | 88 |
| code_repo | 78 |
| filesystem | 68 |
| credential | 31 |
| shell | 27 |

## By WASM class

| Class | Total | Scanned | Suspicious |
|-------|-------|---------|------------|
| unknown | 1 | 1 | 0 |
| wasm-hard | 24 | 24 | 0 |
| wasm-needs-runtime | 8 | 1 | 0 |
| wasm-ready | 2 | 2 | 0 |

## By ecosystem

| Ecosystem | Total | Scanned | Rate |
|-----------|-------|---------|------|
| go | 2 | 2 | 100.0% |
| npm | 25 | 25 | 100.0% |
| python | 8 | 1 | 12.5% |

## Failure categories

| Category | Count |
|----------|-------|
| mcp_start | 7 |

## Cases

| Repo | Tier | OK | Flows | Deps | Stars | Total ms | Error |
|------|------|----|-------|------|-------|----------|-------|
| modelcontextprotocol/servers | tier1 | true | 0 | 4 | 87415 | 13095 |  |
| microsoft/playwright-mcp | tier1 | true | 0 | 5 | 34075 | 1916 |  |
| github/github-mcp-server | tier1 | true | 0 | 2 | 30784 | 30836 |  |
| czlonkowski/n8n-mcp | tier1 | true | 0 | 40 | 21838 | 100981 |  |
| mksglu/context-mode | tier2 | true | 0 | 15 | 17727 | 6858 |  |
| GLips/Figma-Context-MCP | tier1 | true | 0 | 31 | 15148 | 9262 |  |
| BeehiveInnovations/pal-mcp-server | tier1 | false | 0 | 6 | 11597 | 19150 | failed to scan native MCP subject BeehiveInnovations__pal-mcp-server: MCP server closed stdout before JSON response |
| wonderwhy-er/DesktopCommanderMCP | tier1 | true | 0 | 47 | 6184 | 71783 |  |
| executeautomation/mcp-playwright | tier1 | true | 0 | 19 | 5554 | 91836 |  |
| mobile-next/mobile-mcp | tier1 | true | 0 | 26 | 5228 | 8568 |  |
| 21st-dev/magic-mcp | tier1 | true | 0 | 15 | 5166 | 7689 |  |
| exa-labs/exa-mcp-server | tier1 | true | 0 | 17 | 4590 | 6455 |  |
| makenotion/notion-mcp-server | tier1 | true | 0 | 24 | 4437 | 6042 |  |
| Coding-Solo/godot-mcp | tier1 | true | 0 | 5 | 4259 | 3628 |  |
| Pimzino/spec-workflow-mcp | tier1 | true | 0 | 52 | 4234 | 31239 |  |
| zcaceres/markdownify-mcp | tier2 | true | 0 | 6 | 2749 | 34668 |  |
| GongRzhe/Office-Word-MCP-Server | tier1 | true | 0 | 5 | 2057 | 18194 |  |
| GongRzhe/Office-PowerPoint-MCP-Server | tier1 | false | 0 | 4 | 1798 | 13320 | failed to scan native MCP subject GongRzhe__Office-PowerPoint-MCP-Server: MCP server closed stdout before JSON response |
| Flux159/mcp-server-kubernetes | tier1 | true | 0 | 19 | 1438 | 14420 |  |
| bitbonsai/mcpvault | tier2 | true | 0 | 8 | 1432 | 3620 |  |
| designcomputer/mysql_mcp_server | tier1 | false | 0 | 4 | 1304 | 13562 | failed to scan native MCP subject designcomputer__mysql_mcp_server: MCP server closed stdout before JSON response |
| nickclyde/duckduckgo-mcp-server | tier1 | false | 0 | 1 | 1254 | 11195 | failed to scan native MCP subject nickclyde__duckduckgo-mcp-server: MCP server closed stdout before JSON response |
| datalayer/jupyter-mcp-server | tier1 | false | 0 | 7 | 1167 | 29136 | failed to scan native MCP subject datalayer__jupyter-mcp-server: MCP server closed stdout before JSON response |
| strowk/mcp-k8s-go | tier1 | true | 0 | 2 | 382 | 61892 |  |
| smn2gnt/MCP-Salesforce | tier1 | false | 0 | 4 | 179 | 12800 | failed to scan native MCP subject smn2gnt__MCP-Salesforce: MCP server closed stdout before JSON response |
| tsmztech/mcp-server-salesforce | tier1 | true | 0 | 6 | 160 | 7799 |  |
| aashari/mcp-server-atlassian-bitbucket | tier1 | true | 0 | 34 | 156 | 11778 |  |
| aashari/boilerplate-mcp-server | tier1 | true | 0 | 34 | 71 | 10651 |  |
| aashari/mcp-server-atlassian-jira | tier1 | true | 0 | 37 | 71 | 10894 |  |
| HatriGt/hana-mcp-server | tier1 | true | 0 | 4 | 58 | 5846 |  |
| aashari/mcp-server-atlassian-confluence | tier1 | true | 0 | 36 | 58 | 9460 |  |
| dend/brick-mcp-app | tier2 | true | 0 | 22 | 16 | 191921 |  |
| hoangsonww/GitIntel-MCP-Server | tier2 | true | 0 | 7 | 11 | 3322 |  |
| imprvhub/mcp-claude-hackernews | tier2 | true | 0 | 4 | 10 | 2641 |  |
| jhgaylor/hirebase-mcp | tier2 | false | 0 | 3 | 10 | 10444 | failed to scan native MCP subject jhgaylor__hirebase-mcp: MCP server closed stdout before JSON response |
