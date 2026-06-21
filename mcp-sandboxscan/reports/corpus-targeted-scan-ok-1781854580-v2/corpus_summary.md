# Corpus Scan: run-1782068052

- Repos: 35
- Resolved: 35
- Scanned: 33
- Scan success: 94.3%
- Suspicious rate: 36.4% (flows observed, NOT verified vulns)

## Tier-1 (dedicated MCP servers)

- Resolved: 28
- Scanned: 26
- Scan success: 92.9%
- Suspicious rate: 34.6%

- Total latency p50/p95: 3236ms / 14278ms

## Latency (successful scans)

- Count: 33
- Build p50/p95: 2821ms / 13764ms
- Scan p50/p95: 431ms / 7575ms
- Total p50/p95: 3311ms / 25161ms

## Tool metadata and semantic profile

| Subset | Repos with metadata | Tools | Described | Sensitive |
|--------|---------------------|-------|-----------|-----------|
| All | 33 | 502 | 502 | 343 |
| Tier-1 | 26 | 426 | 426 | 299 |
| Tier-2 | 7 | 76 | 76 | 44 |

### Semantic capabilities

| Capability | Tools |
|------------|-------|
| unknown | 159 |
| browser | 156 |
| network | 122 |
| cloud_saas | 100 |
| database | 93 |
| code_repo | 91 |
| filesystem | 74 |
| credential | 30 |
| shell | 28 |

## By WASM class

| Class | Total | Scanned | Suspicious |
|-------|-------|---------|------------|
| unknown | 1 | 1 | 0 |
| wasm-hard | 24 | 22 | 7 |
| wasm-needs-runtime | 8 | 8 | 3 |
| wasm-ready | 2 | 2 | 2 |

## By ecosystem

| Ecosystem | Total | Scanned | Rate |
|-----------|-------|---------|------|
| go | 2 | 2 | 100.0% |
| npm | 25 | 23 | 92.0% |
| python | 8 | 8 | 100.0% |

## Failure categories

| Category | Count |
|----------|-------|
| timeout | 2 |

## Cases

| Repo | Tier | OK | Flows | Deps | Stars | Total ms | Error |
|------|------|----|-------|------|-------|----------|-------|
| modelcontextprotocol/servers | tier1 | true | 1 | 4 | 87415 | 4240 |  |
| microsoft/playwright-mcp | tier1 | true | 0 | 5 | 34075 | 672 |  |
| github/github-mcp-server | tier1 | true | 13 | 2 | 30784 | 885 |  |
| czlonkowski/n8n-mcp | tier1 | true | 2 | 40 | 21838 | 25161 |  |
| mksglu/context-mode | tier2 | true | 0 | 15 | 17727 | 3311 |  |
| GLips/Figma-Context-MCP | tier1 | true | 0 | 31 | 15148 | 4127 |  |
| BeehiveInnovations/pal-mcp-server | tier1 | true | 10 | 6 | 11597 | 4847 |  |
| wonderwhy-er/DesktopCommanderMCP | tier1 | true | 4 | 47 | 6184 | 11105 |  |
| executeautomation/mcp-playwright | tier1 | false | 0 | 19 | 5554 | 93212 | failed to scan native MCP subject executeautomation__mcp-playwright: MCP server timed out after 30s |
| mobile-next/mobile-mcp | tier1 | true | 8 | 26 | 5228 | 2466 |  |
| 21st-dev/magic-mcp | tier1 | false | 0 | 15 | 5166 | 93886 | failed to scan native MCP subject 21st-dev__magic-mcp: MCP server timed out after 30s |
| exa-labs/exa-mcp-server | tier1 | true | 0 | 17 | 4590 | 1289 |  |
| makenotion/notion-mcp-server | tier1 | true | 0 | 24 | 4437 | 1642 |  |
| Coding-Solo/godot-mcp | tier1 | true | 0 | 5 | 4259 | 2267 |  |
| Pimzino/spec-workflow-mcp | tier1 | true | 1 | 52 | 4234 | 14278 |  |
| zcaceres/markdownify-mcp | tier2 | true | 1 | 6 | 2749 | 4416 |  |
| GongRzhe/Office-Word-MCP-Server | tier1 | true | 0 | 5 | 2057 | 3852 |  |
| GongRzhe/Office-PowerPoint-MCP-Server | tier1 | true | 8 | 4 | 1798 | 3508 |  |
| Flux159/mcp-server-kubernetes | tier1 | true | 0 | 19 | 1438 | 4311 |  |
| bitbonsai/mcpvault | tier2 | true | 0 | 8 | 1432 | 1452 |  |
| designcomputer/mysql_mcp_server | tier1 | true | 0 | 4 | 1304 | 3236 |  |
| nickclyde/duckduckgo-mcp-server | tier1 | true | 0 | 1 | 1254 | 3102 |  |
| datalayer/jupyter-mcp-server | tier1 | true | 0 | 7 | 1167 | 10634 |  |
| strowk/mcp-k8s-go | tier1 | true | 8 | 2 | 382 | 1443 |  |
| smn2gnt/MCP-Salesforce | tier1 | true | 0 | 4 | 179 | 3422 |  |
| tsmztech/mcp-server-salesforce | tier1 | true | 0 | 6 | 160 | 3102 |  |
| aashari/mcp-server-atlassian-bitbucket | tier1 | true | 0 | 34 | 156 | 3754 |  |
| aashari/boilerplate-mcp-server | tier1 | true | 0 | 34 | 71 | 3453 |  |
| aashari/mcp-server-atlassian-jira | tier1 | true | 0 | 37 | 71 | 3043 |  |
| HatriGt/hana-mcp-server | tier1 | true | 0 | 4 | 58 | 656 |  |
| aashari/mcp-server-atlassian-confluence | tier1 | true | 0 | 36 | 58 | 2977 |  |
| dend/brick-mcp-app | tier2 | true | 2 | 22 | 16 | 65035 |  |
| hoangsonww/GitIntel-MCP-Server | tier2 | true | 0 | 7 | 11 | 1654 |  |
| imprvhub/mcp-claude-hackernews | tier2 | true | 0 | 4 | 10 | 1291 |  |
| jhgaylor/hirebase-mcp | tier2 | true | 2 | 3 | 10 | 3363 |  |
