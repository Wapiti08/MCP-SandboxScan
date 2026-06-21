# Corpus Scan: run-1781856401

- Repos: 100
- Resolved: 91
- Scanned: 35
- Scan success: 38.5%
- Suspicious rate: 0.0% (flows observed, NOT verified vulns)

## Tier-1 (dedicated MCP servers)

- Resolved: 42
- Scanned: 28
- Scan success: 66.7%
- Suspicious rate: 0.0%

- Total latency p50/p95: 3108ms / 14357ms

## Latency (successful scans)

- Count: 35
- Build p50/p95: 2684ms / 13931ms
- Scan p50/p95: 401ms / 7525ms
- Total p50/p95: 3211ms / 24158ms

## Tool metadata and semantic profile

| Subset | Repos with metadata | Tools | Described | Sensitive |
|--------|---------------------|-------|-----------|-----------|
| All | 35 | 539 | 539 | 418 |
| Tier-1 | 28 | 463 | 463 | 360 |
| Tier-2 | 7 | 76 | 76 | 58 |

### Semantic capabilities

| Capability | Tools |
|------------|-------|
| network | 239 |
| browser | 188 |
| unknown | 121 |
| cloud_saas | 101 |
| database | 95 |
| code_repo | 92 |
| filesystem | 80 |
| credential | 35 |
| shell | 28 |

## By WASM class

| Class | Total | Scanned | Suspicious |
|-------|-------|---------|------------|
| unknown | 2 | 1 | 0 |
| wasm-hard | 49 | 24 | 0 |
| wasm-needs-runtime | 26 | 8 | 0 |
| wasm-ready | 14 | 2 | 0 |

## By ecosystem

| Ecosystem | Total | Scanned | Rate |
|-----------|-------|---------|------|
| go | 11 | 2 | 18.2% |
| npm | 54 | 25 | 46.3% |
| python | 24 | 8 | 33.3% |
| rust | 2 | 0 | 0.0% |

## Failure categories

| Category | Count |
|----------|-------|
| mcp_start | 31 |
| build | 19 |
| timeout | 3 |
| mcp_call | 3 |

## Cases

| Repo | Tier | OK | Flows | Deps | Stars | Total ms | Error |
|------|------|----|-------|------|-------|----------|-------|
| modelcontextprotocol/servers | tier1 | true | 0 | 4 | 87415 | 4065 |  |
| D4Vinci/Scrapling | tier2 | false | 0 | 17 | 64805 | 3175 | failed to scan native MCP subject D4Vinci__Scrapling: MCP server closed stdout before JSON response |
| ruvnet/ruflo | tier2 | false | 0 | 24 | 60130 | 11916 | subject ruvnet__ruflo cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 2"] |
| sansan0/TrendRadar | tier2 | false | 0 | 10 | 59622 | 95990 | failed to scan native MCP subject sansan0__TrendRadar: MCP server closed stdout before JSON response |
| upstash/context7 | tier2 | false | 0 | 12 | 57643 | 10187 | subject upstash__context7 cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| ChromeDevTools/chrome-devtools-mcp | tier2 | false | 0 | 35 | 43945 | 13728 | failed to scan native MCP subject ChromeDevTools__chrome-devtools-mcp: MCP server closed stdout before JSON response |
| bytedance/UI-TARS-desktop | tier2 | false | 0 | 28 | 36836 | 370 | subject bytedance__UI-TARS-desktop cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| microsoft/playwright-mcp | tier1 | true | 0 | 5 | 34075 | 695 |  |
| github/github-mcp-server | tier1 | true | 0 | 2 | 30784 | 1001 |  |
| assafelovic/gpt-researcher | tier2 | false | 0 | 46 | 27776 | 6015 | failed to scan native MCP subject assafelovic__gpt-researcher: MCP server closed stdout before JSON response |
| activepieces/activepieces | tier2 | false | 0 | 319 | 22811 | 64638 | subject activepieces__activepieces cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| czlonkowski/n8n-mcp | tier1 | true | 0 | 40 | 21838 | 24158 |  |
| 1Panel-dev/MaxKB | tier2 | false | 0 | 0 | 21366 | 2999 | subject 1Panel-dev__MaxKB cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| mksglu/context-mode | tier2 | true | 0 | 15 | 17727 | 3308 |  |
| googleapis/mcp-toolbox | tier2 | false | 0 | 3 | 15649 | 3361 | failed to scan native MCP subject googleapis__mcp-toolbox: MCP server closed stdout before JSON response |
| triggerdotdev/trigger.dev | tier2 | false | 0 | 20 | 15391 | 1868 | subject triggerdotdev__trigger.dev cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| GLips/Figma-Context-MCP | tier1 | true | 0 | 31 | 15148 | 3850 |  |
| xpzouying/xiaohongshu-mcp | tier1 | false | 0 | 2 | 14246 | 181804 | failed to scan native MCP subject xpzouying__xiaohongshu-mcp: MCP server closed stdout before JSON response |
| casdoor/casdoor | tier2 | false | 0 | 2 | 13794 | 3320 | failed to scan native MCP subject casdoor__casdoor: MCP server closed stdout before JSON response |
| tadata-org/fastapi_mcp | tier1 | false | 0 | 0 | 11916 | 4056 | failed to scan native MCP subject tadata-org__fastapi_mcp: MCP server closed stdout before JSON response |
| JoeanAmier/XHS-Downloader | tier2 | false | 0 | 13 | 11604 | 94401 | failed to scan native MCP subject JoeanAmier__XHS-Downloader: MCP server closed stdout before JSON response |
| BeehiveInnovations/pal-mcp-server | tier1 | true | 0 | 6 | 11597 | 4293 |  |
| webiny/webiny-js | tier2 | false | 0 | 61 | 7991 | 10359 | subject webiny__webiny-js cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| BrowserMCP/mcp | tier2 | false | 0 | 14 | 6694 | 723 | subject BrowserMCP__mcp cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| firecrawl/firecrawl-mcp-server | tier2 | false | 0 | 6 | 6618 | 2833 | failed to scan native MCP subject firecrawl__firecrawl-mcp-server: tools/call response missing result |
| wonderwhy-er/DesktopCommanderMCP | tier1 | true | 0 | 47 | 6184 | 10573 |  |
| getsentry/XcodeBuildMCP | tier2 | false | 0 | 29 | 5927 | 3113 | failed to scan native MCP subject getsentry__XcodeBuildMCP: MCP server closed stdout before JSON response |
| executeautomation/mcp-playwright | tier1 | true | 0 | 19 | 5554 | 3458 |  |
| nanbingxyz/5ire | tier2 | false | 0 | 117 | 5247 | 47142 | subject nanbingxyz__5ire cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| mobile-next/mobile-mcp | tier1 | true | 0 | 26 | 5228 | 2178 |  |
| 21st-dev/magic-mcp | tier1 | true | 0 | 15 | 5166 | 4110 |  |
| openclaw/Peekaboo | tier2 | false | 0 | 1 | 4730 | 497 | subject openclaw__Peekaboo cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 127"] |
| exa-labs/exa-mcp-server | tier1 | true | 0 | 17 | 4590 | 1134 |  |
| makenotion/notion-mcp-server | tier1 | true | 0 | 24 | 4437 | 1725 |  |
| open-webui/mcpo | tier1 | false | 0 | 0 | 4261 | 4275 | failed to scan native MCP subject open-webui__mcpo: MCP server closed stdout before JSON response |
| Coding-Solo/godot-mcp | tier1 | true | 0 | 5 | 4259 | 2439 |  |
| Pimzino/spec-workflow-mcp | tier1 | true | 0 | 52 | 4234 | 14357 |  |
| haris-musa/excel-mcp-server | tier1 | false | 0 | 0 | 3942 | 4506 | failed to scan native MCP subject haris-musa__excel-mcp-server: MCP server closed stdout before JSON response |
| opensumi/core | tier2 | false | 0 | 72 | 3635 | 13510 | subject opensumi__core cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| av/harbor | tier2 | false | 0 | 1 | 3085 | 756 | failed to scan native MCP subject av__harbor: MCP server closed stdout before JSON response |
| blazickjp/arxiv-mcp-server | tier1 | false | 0 | 14 | 2870 | 3651 | failed to scan native MCP subject blazickjp__arxiv-mcp-server: MCP server closed stdout before JSON response |
| zcaceres/markdownify-mcp | tier2 | true | 0 | 6 | 2749 | 4047 |  |
| brightdata/brightdata-mcp | tier2 | false | 0 | 7 | 2454 | 1782 | failed to scan native MCP subject brightdata__brightdata-mcp: MCP server closed stdout before JSON response |
| GongRzhe/Office-Word-MCP-Server | tier1 | true | 0 | 5 | 2057 | 3706 |  |
| stacklok/toolhive | tier2 | false | 0 | 7 | 1889 | 2121 | subject stacklok__toolhive cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| benborla/mcp-server-mysql | tier2 | false | 0 | 21 | 1834 | 3381 | failed to scan native MCP subject benborla__mcp-server-mysql: MCP server closed stdout before JSON response |
| GongRzhe/Office-PowerPoint-MCP-Server | tier1 | true | 0 | 4 | 1798 | 3405 |  |
| containers/kubernetes-mcp-server | tier2 | false | 0 | 2 | 1700 | 2024 | failed to scan native MCP subject containers__kubernetes-mcp-server: MCP server closed stdout before JSON response |
| f/mcptools | tier2 | false | 0 | 2 | 1595 | 685 | failed to scan native MCP subject f__mcptools: MCP server closed stdout before JSON response |
| datagouv/datagouv-mcp | tier2 | false | 0 | 0 | 1537 | 4458 | failed to scan native MCP subject datagouv__datagouv-mcp: MCP server closed stdout before JSON response |
| MiniMax-AI/MiniMax-MCP | tier1 | false | 0 | 8 | 1515 | 4111 | failed to scan native MCP subject MiniMax-AI__MiniMax-MCP: MCP server closed stdout before JSON response |
| Flux159/mcp-server-kubernetes | tier1 | true | 0 | 19 | 1438 | 4230 |  |
| qdrant/mcp-server-qdrant | tier1 | false | 0 | 0 | 1438 | 18060 | subject qdrant__mcp-server-qdrant cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| bitbonsai/mcpvault | tier2 | true | 0 | 8 | 1432 | 1800 |  |
| designcomputer/mysql_mcp_server | tier1 | true | 0 | 4 | 1304 | 3088 |  |
| mixelpixx/KiCAD-MCP-Server | tier1 | false | 0 | 15 | 1299 | 6676 | failed to scan native MCP subject mixelpixx__KiCAD-MCP-Server: MCP server closed stdout before JSON response |
| nickclyde/duckduckgo-mcp-server | tier1 | true | 0 | 1 | 1254 | 3035 |  |
| datalayer/jupyter-mcp-server | tier1 | true | 0 | 7 | 1167 | 10335 |  |
| GongRzhe/Gmail-MCP-Server | tier1 | false | 0 | 13 | 1146 | 6743 | failed to scan native MCP subject GongRzhe__Gmail-MCP-Server: MCP server closed stdout before JSON response |
| neka-nat/freecad-mcp | tier1 | false | 0 | 0 | 1140 | 3845 | failed to scan native MCP subject neka-nat__freecad-mcp: MCP server closed stdout before JSON response |
| mongodb-js/mongodb-mcp-server | tier1 | false | 0 | 91 | 1055 | 2002 | subject mongodb-js__mongodb-mcp-server cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| nduckmink/arkon | tier2 | false | 0 | 4 | 997 | 4794 | subject nduckmink__arkon cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 1"] |
| patruff/ollama-mcp-bridge | tier1 | false | 0 | 13 | 975 | 867 | subject patruff__ollama-mcp-bridge cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 126"] |
| negokaz/excel-mcp-server | tier1 | false | 0 | 2 | 974 | 368 | subject negokaz__excel-mcp-server cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 127"] |
| ravitemer/mcp-hub | tier2 | false | 0 | 19 | 495 | 270887 | failed to scan native MCP subject ravitemer__mcp-hub: MCP server timed out after 90s |
| strowk/mcp-k8s-go | tier1 | true | 0 | 2 | 382 | 1816 |  |
| smn2gnt/MCP-Salesforce | tier1 | true | 0 | 4 | 179 | 3108 |  |
| tsmztech/mcp-server-salesforce | tier1 | true | 0 | 6 | 160 | 2877 |  |
| aashari/mcp-server-atlassian-bitbucket | tier1 | true | 0 | 34 | 156 | 3520 |  |
| its-dart/dart-mcp-server | tier2 | false | 0 | 9 | 128 | 2888 | failed to scan native MCP subject its-dart__dart-mcp-server: MCP server closed stdout before JSON response |
| jhgaylor/node-candidate-mcp-server | tier2 | false | 0 | 7 | 81 | 1307 | failed to scan native MCP subject jhgaylor__node-candidate-mcp-server: MCP server closed stdout before JSON response |
| aashari/boilerplate-mcp-server | tier1 | true | 0 | 34 | 71 | 3417 |  |
| aashari/mcp-server-atlassian-jira | tier1 | true | 0 | 37 | 71 | 2936 |  |
| HatriGt/hana-mcp-server | tier1 | true | 0 | 4 | 58 | 636 |  |
| aashari/mcp-server-atlassian-confluence | tier1 | true | 0 | 36 | 58 | 2765 |  |
| kocierik/mcp-nomad | tier2 | false | 0 | 2 | 52 | 727 | failed to scan native MCP subject kocierik__mcp-nomad: MCP server closed stdout before JSON response |
| seehiong/blender-mcp-n8n | tier2 | false | 0 | 7 | 41 | 2702 | failed to scan native MCP subject seehiong__blender-mcp-n8n: MCP server closed stdout before JSON response |
| flaviodelgrosso/fastify-mcp-server | tier2 | false | 0 | 24 | 27 | 2267 | failed to scan native MCP subject flaviodelgrosso__fastify-mcp-server: MCP server closed stdout before JSON response |
| bsmi021/mcp-chain-of-draft-server | tier2 | false | 0 | 6 | 24 | 1353 | failed to scan native MCP subject bsmi021__mcp-chain-of-draft-server: tools/call response missing result |
| philogicae/torrent-search-mcp | tier1 | false | 0 | 0 | 23 | 3428 | failed to scan native MCP subject philogicae__torrent-search-mcp: MCP server closed stdout before JSON response |
| dend/brick-mcp-app | tier2 | true | 0 | 22 | 16 | 185224 |  |
| xgd16/ai-mcp | tier2 | false | 0 | 2 | 14 | 271980 | failed to scan native MCP subject xgd16__ai-mcp: MCP server timed out after 90s |
| dirmacs/ares | tier2 | false | 0 | 62 | 13 | 7099 | subject dirmacs__ares cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 101"] |
| hoangsonww/GitIntel-MCP-Server | tier2 | true | 0 | 7 | 11 | 1860 |  |
| Typewise/mcp-chaos-rig | tier2 | false | 0 | 11 | 10 | 271398 | failed to scan native MCP subject Typewise__mcp-chaos-rig: MCP server timed out after 90s |
| dryas/mail-shadow-mcp | tier2 | false | 0 | 2 | 10 | 2216 | failed to scan native MCP subject dryas__mail-shadow-mcp: MCP server closed stdout before JSON response |
| imprvhub/mcp-claude-hackernews | tier2 | true | 0 | 4 | 10 | 1432 |  |
| jhgaylor/hirebase-mcp | tier2 | true | 0 | 3 | 10 | 3211 |  |
| nullablevariant/rust-mcp-core | tier2 | false | 0 | 25 | 10 | 537 | failed to scan native MCP subject nullablevariant__rust-mcp-core: failed to spawn MCP server `target/release/rust-mcp-core`: No such file or directory (os error 2) |
| nobrainer-tech/langflow-mcp | tier2 | false | 0 | 13 | 9 | 2506 | failed to scan native MCP subject nobrainer-tech__langflow-mcp: tools/call response missing result |
| sotayamashita/openapi-mcp-server | tier2 | false | 0 | 22 | 7 | 624 | subject sotayamashita__openapi-mcp-server cannot be scanned: status=Failed, blockers=["build command exited with status exit status: 127"] |
