//! Built-in plugin marketplace catalog (same set as official xAI index + Nur extras).

/// A marketplace entry. Installed via git into `~/.nur/plugins/<id>/`.
#[derive(Debug, Clone, Copy)]
pub struct PluginEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    /// Git clone URL (https).
    pub source_url: &'static str,
    /// Optional subdirectory inside the repo that is the plugin root.
    pub path_in_repo: Option<&'static str>,
}

/// Full catalog shown in `/plugins`.
pub const CATALOG: &[PluginEntry] = &[
    PluginEntry {
        id: "superpowers",
        name: "Superpowers",
        description: "TDD, systematic debugging, collaboration patterns, engineering workflows",
        category: "development",
        source_url: "https://github.com/obra/superpowers.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "vercel",
        name: "Vercel",
        description: "Deploy, env vars, Next.js, AI SDK, Marketplace — Vercel platform skills",
        category: "deployment",
        source_url: "https://github.com/vercel/vercel-plugin.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "chrome-devtools",
        name: "Chrome DevTools",
        description: "Live Chrome control: network, console, performance traces, automation",
        category: "development",
        source_url: "https://github.com/ChromeDevTools/chrome-devtools-mcp.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "firecrawl",
        name: "Firecrawl",
        description: "Scrape, crawl, and search the web into clean LLM-ready markdown",
        category: "development",
        source_url: "https://github.com/firecrawl/firecrawl-grok-plugin.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "figma",
        name: "Figma",
        description: "Design-to-code: read Figma context, Code Connect, canvas write",
        category: "development",
        source_url: "https://github.com/figma/mcp-server-guide.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "sentry",
        name: "Sentry",
        description: "Error monitoring: issues, stack traces, production debug",
        category: "monitoring",
        source_url: "https://github.com/getsentry/plugin-grok.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "cloudflare",
        name: "Cloudflare",
        description: "Workers, Durable Objects, Wrangler, MCP servers, web performance",
        category: "development",
        source_url: "https://github.com/cloudflare/skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "mongodb",
        name: "MongoDB",
        description: "Database explore, collections, queries, Atlas best practices",
        category: "database",
        source_url: "https://github.com/mongodb/agent-skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "axiom",
        name: "Axiom",
        description: "Logs/metrics with APL, SRE investigations, monitors, cost analysis",
        category: "observability",
        source_url: "https://github.com/axiomhq/skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "railway",
        name: "Railway",
        description: "Deploy services, DBs, env vars, domains, metrics on Railway",
        category: "deployment",
        source_url: "https://github.com/railwayapp/railway-skills.git",
        path_in_repo: Some("plugins/railway"),
    },
    PluginEntry {
        id: "fable",
        name: "Fable",
        description: "Think / act / prove workflow: fable-method, fable-loop, fable-judge",
        category: "development",
        source_url: "https://github.com/Sahir619/fable-method.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "impeccable",
        name: "Impeccable",
        description: "Design language for AI harnesses: audit / polish / critique / animate, 46 detector rules",
        category: "development",
        source_url: "https://github.com/pbakaus/impeccable.git",
        path_in_repo: Some("plugin"),
    },
    // ── Curated skill packs (Agent Skills / SKILL.md format) ──────────────
    PluginEntry {
        id: "mattpocock",
        name: "Matt Pocock Skills",
        description: "Real-engineering skills: grill-me, triage, tdd, to-spec, implement, handoff (skills for real engineers)",
        category: "development",
        source_url: "https://github.com/mattpocock/skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "addyosmani",
        name: "Addy Osmani Agent Skills",
        description: "Production engineering: context engineering, frontend UI, security, shipping, TDD, code review",
        category: "development",
        source_url: "https://github.com/addyosmani/agent-skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "builderio",
        name: "Builder.io Skills",
        description: "Agent efficiency: efficient-fable, plan-arbiter, stay-within-limits, visual-plan, read-the-damn-docs",
        category: "development",
        source_url: "https://github.com/BuilderIO/skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "mengto",
        name: "Meng To Skills",
        description: "Design + web craft: UI prompting, motion systems, brand worlds, capture/perf skills",
        category: "design",
        source_url: "https://github.com/MengTo/Skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "google-skills",
        name: "Google Skills",
        description: "Official Google product skills (Ads, Analytics, Cloud, Firebase, Gemini, …)",
        category: "platform",
        source_url: "https://github.com/google/skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "nvidia-skills",
        name: "NVIDIA Skills",
        description: "CUDA, cuOpt, accelerated computing, AIQ research/deploy (300 skill packs)",
        category: "platform",
        source_url: "https://github.com/NVIDIA/skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "scientific",
        name: "Scientific Agent Skills",
        description: "K-Dense AI scientist pack: biopython, astropy, benchling, paper search, lab tooling",
        category: "science",
        source_url: "https://github.com/K-Dense-AI/scientific-agent-skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "ai-marketing",
        name: "AI Marketing Skills",
        description: "Growth, SEO ops, content ops, outbound, sales pipeline, clone-site (ericosiu)",
        category: "marketing",
        source_url: "https://github.com/ericosiu/ai-marketing-skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "finance-skills",
        name: "Finance Skills",
        description: "Financial analysis: valuation, earnings, options, market data readers (himself65)",
        category: "finance",
        source_url: "https://github.com/himself65/finance-skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "longbridge",
        name: "Longbridge Skills",
        description: "Markets: portfolio, quant, technicals, earnings, value investing, watchlist",
        category: "finance",
        source_url: "https://github.com/longbridge/skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "buffett",
        name: "Buffett Skills",
        description: "Value-investing skill pack built on Warren Buffett principles (agi-now)",
        category: "finance",
        source_url: "https://github.com/agi-now/buffett-skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "cre-skills",
        name: "CRE Agent Skills",
        description: "Commercial real estate: underwriting, due diligence, financing, brokerage",
        category: "finance",
        source_url: "https://github.com/ahacker-1/cre-agent-skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "claude-skills-mega",
        name: "Claude Skills Mega Pack",
        description: "Large multi-domain pack (business, agents, growth, ops) — install selectively; large download",
        category: "catalog",
        source_url: "https://github.com/alirezarezvani/claude-skills.git",
        path_in_repo: None,
    },
    PluginEntry {
        id: "journal-skills",
        name: "Awesome Journal Skills",
        description: "Academic journal skill packs (AAAI, ACL, AEJ, …) — huge; use for paper workflows only",
        category: "science",
        source_url: "https://github.com/brycewang-stanford/Awesome-Journal-Skills.git",
        path_in_repo: None,
    },
];

pub fn catalog() -> &'static [PluginEntry] {
    CATALOG
}

pub fn by_id(id: &str) -> Option<&'static PluginEntry> {
    let id = id.trim();
    CATALOG.iter().find(|p| p.id.eq_ignore_ascii_case(id))
}
