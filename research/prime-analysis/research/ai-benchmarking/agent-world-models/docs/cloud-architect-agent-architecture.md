# Cloud Architecture Agent: Full Workflow Design

**Date:** 2026-05-22  
**Organization:** AMN Healthcare  
**Purpose:** Design document for autonomous cloud architecture agent system

---

## Executive Summary

Building a cloud architecture agent requires three layers:
1. **Integration Layer** - MCP servers connecting to infrastructure systems
2. **Intelligence Layer** - Skills, agents, and memory for decision-making
3. **Automation Layer** - Hooks and workflows for autonomous operation

---

## Current Infrastructure Landscape

### Systems to Integrate
- **Cloud Platforms**: Azure (primary), AWS (secondary)
- **Code & Version Control**: Azure DevOps, GitHub
- **Ticketing**: ServiceNow (SNOW), ADO work items
- **Observability**: New Relic
- **Custom**: Internal MCPs from other teams

### Data Flow Patterns
```
Ticket (SNOW/ADO) → Architecture Decision → Infrastructure Change → 
Observability (New Relic) → Feedback Loop → Update Ticket
```

---

## Architecture: Three-Phase Build

### Phase 1: Foundation (Weeks 1-4)
**Goal:** Basic integration and manual workflows

**MCP Servers Required:**
1. **Azure MCP** - Resource management, ARM templates, Azure CLI integration
2. **GitHub MCP** - Already available as plugin, repository operations
3. **New Relic MCP** - Query NRQL, dashboards, alerts, incident management
4. **Filesystem MCP** - Local IaC repository management

**Configuration Structure:**
```
~/.claude/
├── settings.json           # Global permissions, MCP server configs
└── projects/
    └── infrastructure/
        ├── .claude/
        │   ├── CLAUDE.md   # Architecture context, conventions
        │   └── settings.json # Project-specific permissions
        └── memory/
            ├── architecture_decisions.md
            ├── runbooks.md
            └── incident_patterns.md
```

**Skills to Create:**
- `infra-check` - Health check across Azure resources via New Relic
- `cost-analyze` - Azure cost analysis and recommendations
- `deploy-review` - Pre-deployment validation against architecture standards

**Memory Categories:**
- Architecture decisions (why certain patterns chosen)
- Incident patterns (what broke, why, how fixed)
- Resource relationships (dependencies between services)
- Team conventions (naming, tagging, deployment windows)

### Phase 2: Integration (Weeks 5-12)
**Goal:** Cross-system workflows and intelligent routing

**Additional MCP Servers:**
1. **Azure DevOps MCP** - Work items, pipelines, repos (may need custom build)
2. **ServiceNow MCP** - Incident/change tickets (may need custom build)
3. **AWS MCP** - EC2, S3, Lambda, CloudWatch integration
4. **Custom Team MCPs** - Integration with other groups' MCP servers

**Advanced Configuration:**
```
settings.json structure:
{
  "mcpServers": {
    "azure": {
      "command": "npx",
      "args": ["-y", "@azure/mcp-server"],
      "env": {
        "AZURE_SUBSCRIPTION_ID": "...",
        "AZURE_TENANT_ID": "..."
      }
    },
    "newrelic": {
      "command": "node",
      "args": ["/path/to/newrelic-mcp/index.js"],
      "env": {
        "NEW_RELIC_API_KEY": "...",
        "NEW_RELIC_ACCOUNT_ID": "..."
      }
    },
    "servicenow": {
      "command": "python",
      "args": ["/path/to/snow-mcp/server.py"],
      "env": {
        "SNOW_INSTANCE": "amnhealthcare.service-now.com",
        "SNOW_API_KEY": "..."
      }
    },
    "azure-devops": {
      "command": "node",
      "args": ["/path/to/ado-mcp/index.js"],
      "env": {
        "ADO_ORG": "amnhealthcare",
        "ADO_PAT": "..."
      }
    }
  },
  "hooks": {
    "beforeToolCall": {
      "Edit": "validate-terraform-syntax",
      "Write": "check-secrets-scan"
    },
    "afterToolCall": {
      "Bash": "log-command-audit"
    }
  },
  "permissions": {
    "global": {
      "allow": [
        "Read:**/*.tf",
        "Read:**/*.yaml",
        "Bash:terraform plan",
        "Bash:az account show"
      ]
    }
  }
}
```

**Skills for Cross-System Workflows:**
- `ticket-to-infra` - Take SNOW/ADO ticket → validate → create branch → make changes → PR
- `alert-to-ticket` - New Relic alert → create SNOW incident → gather context → suggest fix
- `deploy-pipeline` - Coordinate deployment across ADO pipeline → monitor in New Relic → update ticket
- `cost-ticket` - Analyze cost spike → create optimization ticket → estimate savings

**Agent Definitions:**
Create specialized agents in `.claude/agents/`:
- `cloud-reviewer.md` - Reviews IaC changes for Azure/AWS best practices
- `incident-responder.md` - Pulls data from New Relic, suggests remediation, updates tickets
- `cost-optimizer.md` - Analyzes spend, right-sizes resources, creates action tickets

### Phase 3: Autonomy (Weeks 13+)
**Goal:** Self-sufficient agent-to-agent architecture

**Autonomous Workflows:**
1. **Self-Healing**: New Relic alert → Incident agent assesses → Auto-remediate or escalate → Update SNOW
2. **Proactive Optimization**: Daily cost analysis → Identify waste → Create optimization PRs → Assign to teams
3. **Architecture Governance**: PR created → Cloud reviewer validates → Request changes or approve → Log decision
4. **Capacity Planning**: Monitor trends → Predict capacity needs → Create scaling tickets → Propose solutions

**Agent Communication Protocol:**
```yaml
# Example: Incident agent → Cloud architect agent interaction
interaction_pattern:
  trigger: new_relic_alert
  primary_agent: incident-responder
  consultation:
    - agent: cloud-architect
      question: "Is this architecture pattern expected? Review history."
      context: [alert_data, affected_resources, recent_changes]
    - agent: cost-optimizer
      question: "What's the cost impact of proposed remediation?"
  decision: 
    authority: incident-responder
    approval_required: true (for destructive actions)
  tracking:
    - Create SNOW incident
    - Update ADO work item
    - Log to architecture memory
```

**Memory System Evolution:**
```
memory/
├── architecture/
│   ├── decisions/
│   │   ├── 2026-05-azure-network-topology.md
│   │   └── 2026-04-multi-region-strategy.md
│   ├── patterns/
│   │   ├── microservices-deployment.md
│   │   └── disaster-recovery.md
│   └── incidents/
│       ├── 2026-05-22-database-latency.md
│       └── 2026-05-15-storage-capacity.md
├── operational/
│   ├── runbooks/
│   ├── cost-baselines/
│   └── sla-tracking/
└── organizational/
    ├── team-ownership.md
    ├── escalation-paths.md
    └── compliance-requirements.md
```

---

## Technical Requirements

### MCP Server Availability Assessment

**Likely Available (or in plugins):**
- ✅ GitHub - Available as plugin
- ✅ Terraform - Community MCP servers exist
- ⚠️ Azure - May exist, need to verify
- ⚠️ AWS - May exist, need to verify  
- ⚠️ New Relic - May need custom build

**Need Custom Development:**
- ❌ Azure DevOps - Custom MCP required
- ❌ ServiceNow - Custom MCP required
- ❌ Internal systems - Custom MCPs required

### Custom MCP Development Priority
1. **ServiceNow MCP** (High) - Critical for ticket workflow
   - Read/create/update incidents and change requests
   - Query ticket history
   - Update work notes

2. **Azure DevOps MCP** (High) - Code and work item integration
   - Work items CRUD
   - Pipeline triggering and monitoring
   - Repository operations (if not using GitHub)

3. **New Relic MCP** (Medium-High) - Observability integration
   - NRQL query execution
   - Alert management
   - Dashboard retrieval
   - Incident correlation

4. **Azure/AWS MCPs** (Medium) - If not available
   - Resource enumeration
   - Cost queries
   - Configuration retrieval
   - Read-only initially, write operations later

### Security Considerations

**Credential Management:**
- MCP servers should use environment variables or secret stores
- Never commit credentials to CLAUDE.md or skills
- Use Azure Key Vault / AWS Secrets Manager
- Implement credential rotation

**Permissions Model:**
- Start read-only for all MCPs
- Require human approval for destructive operations
- Use hooks to enforce approval gates
- Log all actions for audit

**Autonomous Operations Safety:**
```json
{
  "autonomy_levels": {
    "level_0_read_only": {
      "allowed": ["query", "read", "analyze"],
      "approval": "none"
    },
    "level_1_safe_writes": {
      "allowed": ["create_branch", "create_ticket", "update_ticket_notes"],
      "approval": "automatic"
    },
    "level_2_impactful": {
      "allowed": ["create_pr", "modify_resources", "trigger_pipeline"],
      "approval": "required",
      "notification": "slack_webhook"
    },
    "level_3_destructive": {
      "allowed": ["delete_resources", "force_push", "production_deploy"],
      "approval": "multi_party",
      "notification": "page_oncall"
    }
  }
}
```

---

## Practical Implementation Path

### Week 1: Foundation Setup
1. Install GitHub plugin
2. Set up project structure with CLAUDE.md
3. Create initial memory structure
4. Document architecture standards in CLAUDE.md
5. Create first skill: `infra-status` (read-only health check)

### Week 2-3: MCP Assessment & Development
1. Research available Azure/AWS/New Relic MCPs
2. Build prototype ServiceNow MCP (read-only)
3. Build prototype Azure DevOps MCP (read-only)
4. Test basic cross-system queries

### Week 4: Integration Testing
1. Configure all MCP servers in settings.json
2. Create skills for common workflows
3. Test ticket → code → observability flow
4. Refine permissions and hooks

### Week 5-8: Advanced Workflows
1. Build agent definitions for specialized tasks
2. Create cross-system workflow skills
3. Implement memory patterns for decision tracking
4. Add automation hooks for repetitive tasks

### Week 9-12: Multi-Agent Coordination
1. Define agent communication protocols
2. Implement agent-to-agent consultation patterns
3. Build decision logging system
4. Create escalation workflows

### Week 13+: Autonomous Operations
1. Implement self-healing workflows
2. Add proactive monitoring and optimization
3. Build agent authority framework
4. Establish governance and audit trails

---

## Success Metrics

### Phase 1 Metrics
- Time to query infrastructure status: <2 min
- Manual steps eliminated: 5-10 common operations
- Documentation accuracy: 90%+ match to actual state

### Phase 2 Metrics
- Ticket → Resolution cycle time: -30%
- Cross-system context switches: -50%
- Architecture decision documentation: 100% coverage

### Phase 3 Metrics
- Self-healing success rate: >80%
- Mean time to detection: <5 min
- Mean time to resolution: -40%
- Cost optimization savings: measurable ROI

---

## Risk Mitigation

### Technical Risks
- **MCP reliability**: Build fallback mechanisms, cache critical data
- **API rate limits**: Implement queuing, respect limits
- **Authentication failures**: Automatic retry with alerting

### Operational Risks
- **Autonomous errors**: Comprehensive logging, easy rollback
- **Security breaches**: Least privilege, audit all actions
- **Agent conflicts**: Clear authority hierarchy, escalation paths

### Organizational Risks
- **Team adoption**: Training, documentation, gradual rollout
- **Process changes**: Align with ITIL/ITSM frameworks
- **Compliance**: Audit trails, approval workflows, change control

---

## Next Steps

See "Getting Started" section in summary for immediate actions.
