// Real UOR-R4 V13 Qwen3-0.6B + 16-Tool Foundry Webharness Agent Loop.
// System prompt matches Run 5 LoRA training (`foundry_schemas.json`) verbatim so
// WASM KV-prefix cache hits across multi-turn conversations and tool calls.

export const FOUNDRY_SYSTEM_PROMPT =
  'You are UOR-R4 Foundry Assistant inside a browser virtual workstation with Studio workspace files, a real Linux 6.18.7 VM (9p mounted at /workspace), a secondary RV64IMAC MVM sandbox, GitHub REST/Git Data tools, PrismPM capsules, LexLean 0.3.0 verification, UOR Standards, and Stitch P2P/fetch networking.\n' +
  'When answering general questions or explaining concepts in Chat mode, respond directly in clear concise markdown without calling tools.\n' +
  'When editing or creating app files in Build mode, emit <prism-write path="relative/path" description="short summary">COMPLETE FILE CONTENT</prism-write>.\n' +
  'When executing workspace, VM, GitHub, PrismPM, LexLean, Standards, or Stitch actions, emit a single JSON call inside <tool_call>{"name": "...", "arguments": {...}}</tool_call>.\n' +
  'Available tools: workspace_read(path, start_line?, end_line?), workspace_write(path, content), workspace_list(dir), linux_exec(command, cwd?), mvm_exec(command), github_status(repo?), github_list_repos(owner?), github_clone(repo, branch?), github_pull(repo, branch?), github_push(repo, branch, message), prismpm_verify(target, action?), lexlean_check(path), lexlean_verify(path, profile?), standards_lookup(query), stitch_fetch(url, method?), stitch_peer_status(peer_id?).';

export const FOUNDRY_TOOL_NAMES = Object.freeze([
  'workspace_read',
  'workspace_write',
  'workspace_list',
  'linux_exec',
  'mvm_exec',
  'github_status',
  'github_list_repos',
  'github_clone',
  'github_pull',
  'github_push',
  'prismpm_verify',
  'lexlean_check',
  'lexlean_verify',
  'standards_lookup',
  'stitch_fetch',
  'stitch_peer_status',
]);

const MAX_TOOL_ROUNDS = 6;

export function parseToolCall(text) {
  const src = String(text || '');
  const match = src.match(/<tool_call>\s*([\s\S]*?)\s*(?:<\/tool_call>|$)/);
  if (!match) return null;
  const raw = match[1].trim();
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw);
    const name = String(parsed.name || parsed.tool || '').trim();
    if (!name) return null;
    const args =
      parsed.arguments && typeof parsed.arguments === 'object' && !Array.isArray(parsed.arguments)
        ? parsed.arguments
        : {};
    // Normalize aliases only if older prompt syntax appears.
    if (name === 'bash' || name === 'shell' || name === 'exec' || name === 'terminal') {
      return { name: 'linux_exec', arguments: { command: String(args.command || args.cmd || parsed.command || '') } };
    }
    if (name === 'mvm') {
      return { name: 'mvm_exec', arguments: { command: String(args.command || args.cmd || parsed.command || '') } };
    }
    return { name, arguments: args };
  } catch {
    return null;
  }
}

export async function dispatchFoundryTool(call, tools) {
  const { name, arguments: args } = call;
  if (!FOUNDRY_TOOL_NAMES.includes(name)) {
    return { ok: false, error: `Unknown tool: ${name}` };
  }
  const fn = tools?.[name];
  if (typeof fn !== 'function') {
    return { ok: false, error: `Tool ${name} is not wired in this context` };
  }
  try {
    const result = await fn(args || {});
    return result && typeof result === 'object' ? result : { ok: true, result };
  } catch (err) {
    return { ok: false, error: String(err?.message || err) };
  }
}

export async function runUorR4AgentTurn({
  appId = null,
  chatId,
  modelId,
  messages,
  userPrompt,
  chatMode = 'build',
  maxNewTokens = 384,
  signal,
  uorR4Runtime,
  mvmHost,
  tools = {},
  onText,
}) {
  const prompt = String(userPrompt || '').trim();

  // Explicit user slash-command runs RV64IMAC MVM or Linux VM directly.
  const directMatch = prompt.match(/^\/(mvm|linux|bash|sh)\s+([\s\S]+)$/i);
  if (directMatch) {
    const kind = directMatch[1].toLowerCase();
    const cmd = directMatch[2].trim();
    if (kind !== 'mvm' && typeof tools.linux_exec === 'function') {
      const res = await tools.linux_exec({ command: cmd });
      const body = [
        `**Linux 6.18.7 Shell** (\`${cmd}\`) · exit \`${res.exit_code ?? 0}\` · cwd \`${res.cwd || '/workspace'}\``,
        res.stdout ? `\`\`\`\n${String(res.stdout).trimEnd()}\n\`\`\`` : '',
        res.stderr ? `**stderr:**\n\`\`\`\n${String(res.stderr).trimEnd()}\n\`\`\`` : '',
      ]
        .filter(Boolean)
        .join('\n\n');
      await onText(body);
      return {
        fullText: body,
        toolExecutions: 1,
        updatedFiles: Boolean(res.updatedFiles),
        weightsMatched: true,
      };
    }
    const res = await mvmHost.execCommand(appId, cmd, { echoToSession: true });
    const body = [
      `**MVM RV64IMAC Shell** (\`${cmd}\`) · exit \`${res.exit_code}\` · retired \`${res.instructions_retired}\` · elf \`${res.compiled_elf_machine}\` · cwd \`${res.cwd}\``,
      res.stdout ? `\`\`\`\n${res.stdout.trimEnd()}\n\`\`\`` : '',
      res.stderr ? `**stderr:**\n\`\`\`\n${res.stderr.trimEnd()}\n\`\`\`` : '',
      `CoW SHA-256: \`${res.cow_state_sha256}\``,
    ]
      .filter(Boolean)
      .join('\n\n');
    await onText(body);
    return {
      fullText: body,
      toolExecutions: 1,
      updatedFiles: false,
      weightsMatched: true,
    };
  }

  let transcript = '';
  let turnMessages = Array.isArray(messages) ? [...messages] : [];
  let weightsMatched = true;
  let toolExecutions = 0;
  let updatedFiles = false;
  const isV13 = String(modelId || '').includes('qwen3-0.6b-foundry');

  for (let round = 0; round <= MAX_TOOL_ROUNDS; round += 1) {
    const roundPrefix = transcript;
    const gen = await uorR4Runtime.generate({
      modelId,
      chatId: `${chatId}-r${round}`,
      systemPrompt: isV13 ? FOUNDRY_SYSTEM_PROMPT : '',
      prompt: round === 0 ? prompt : turnMessages[turnMessages.length - 1].content,
      messages: round === 0 ? turnMessages : turnMessages.slice(0, -1),
      maxNewTokens: isV13 ? maxNewTokens : 48,
      stopSequences: isV13 ? ['</tool_call>'] : undefined,
      signal,
      onToken: ({ fullText }) => {
        void onText(roundPrefix + fullText);
      },
    });
    if (!gen.weightsMatched) {
      weightsMatched = false;
    }
    const cleanOut = String(gen.fullText || '').trim();
    transcript = roundPrefix + cleanOut;
    await onText(transcript);

    const call = parseToolCall(cleanOut);
    if (!call || round === MAX_TOOL_ROUNDS) {
      break;
    }
    // In Home Chat mode, skip mutating app-workspace tools cleanly if called accidentally.
    if (chatMode === 'chat' && call.name === 'workspace_write') {
      const refusal = { ok: false, error: 'Home Chat is read-only. Click "Build this in Studio" to write files.' };
      const responseBlock = `\n<tool_response>${JSON.stringify(refusal)}</tool_response>\n`;
      transcript += responseBlock;
      await onText(transcript);
      const basePrior =
        round === 0
          ? [...turnMessages, { role: 'user', content: prompt }]
          : turnMessages;
      turnMessages = [
        ...basePrior,
        { role: 'assistant', content: cleanOut },
        { role: 'user', content: `<tool_response>${JSON.stringify(refusal)}</tool_response>` },
      ];
      continue;
    }

    toolExecutions += 1;
    const result = await dispatchFoundryTool(call, tools);
    if (result?.updatedFiles) {
      updatedFiles = true;
    }
    const compact = JSON.stringify(result);
    const bounded = compact.length > 6000 ? compact.slice(0, 6000) + '..."}' : compact;
    const responseBlock = `\n<tool_response>${bounded}</tool_response>\n`;
    transcript += responseBlock;
    await onText(transcript);

    const basePrior =
      round === 0
        ? [...turnMessages, { role: 'user', content: prompt }]
        : turnMessages;
    turnMessages = [
      ...basePrior,
      { role: 'assistant', content: cleanOut },
      { role: 'user', content: `<tool_response>${bounded}</tool_response>` },
    ];
  }

  return {
    fullText: transcript,
    toolExecutions,
    updatedFiles,
    weightsMatched,
  };
}
