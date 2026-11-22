// Real UOR-R4 + RV64IMAC MVM agent loop for Foundry chat turns.
//
// Normal prompts always execute real `uor-r4-integer` WASM via `uorR4Runtime.generate(...)`.
// Explicit `/mvm <cmd>` or `/bash <cmd>` turns invoke `WasmMvmSandbox` directly and report the
// measured RV64IMAC output and state hash without misattributing it to model weights.

const MAX_TOOL_ROUNDS = 4;

export function parseToolCall(text) {
  const match = String(text || '').match(/<tool_call>\s*([\s\S]*?)\s*<\/tool_call>/);
  if (!match) return null;
  const raw = match[1].trim();
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw);
    const name = String(parsed.name || parsed.tool || 'mvm').toLowerCase();
    if (!['bash', 'shell', 'mvm', 'exec', 'terminal'].includes(name)) return null;
    const args = parsed.arguments && typeof parsed.arguments === 'object' ? parsed.arguments : parsed;
    const command = String(args.command || args.cmd || '').trim();
    return command ? { name, command } : null;
  } catch {
    return raw ? { name: 'mvm', command: raw } : null;
  }
}

export async function runUorR4AgentTurn({
  appId,
  chatId,
  modelId,
  messages,
  userPrompt,
  signal,
  uorR4Runtime,
  mvmHost,
  onText,
}) {
  const prompt = String(userPrompt || '').trim();

  // Explicit user slash-command runs RV64IMAC MVM directly (clearly labeled as MVM shell output).
  const directMatch = prompt.match(/^\/(?:mvm|bash|sh)\s+([\s\S]+)$/i);
  if (directMatch) {
    const cmd = directMatch[1].trim();
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
      weightsMatched: true,
    };
  }

  let transcript = '';
  let turnMessages = [...messages];
  let weightsMatched = true;
  let toolExecutions = 0;

  for (let round = 0; round <= MAX_TOOL_ROUNDS; round += 1) {
    const roundPrefix = transcript;
    const gen = await uorR4Runtime.generate({
      modelId,
      chatId: `${chatId}-r${round}`,
      prompt: round === 0 ? prompt : turnMessages[turnMessages.length - 1].content,
      messages: turnMessages,
      maxNewTokens: 48,
      signal,
      onToken: ({ fullText }) => {
        void onText(roundPrefix + fullText);
      },
    });
    if (!gen.weightsMatched) {
      weightsMatched = false;
    }
    transcript = roundPrefix + gen.fullText;
    await onText(transcript);

    const call = parseToolCall(gen.fullText);
    if (!call || round === MAX_TOOL_ROUNDS) {
      break;
    }
    toolExecutions += 1;
    const res = await mvmHost.execCommand(appId, call.command, { echoToSession: true });
    const responseJson = JSON.stringify({
      stdout: res.stdout,
      stderr: res.stderr,
      exit_code: res.exit_code,
      cwd: res.cwd,
      instructions_retired: res.instructions_retired,
      compiled_elf_machine: res.compiled_elf_machine,
      cow_state_sha256: res.cow_state_sha256,
    });
    const responseBlock = `\n<tool_response>${responseJson}</tool_response>\n`;
    transcript += responseBlock;
    await onText(transcript);
    turnMessages = [
      ...turnMessages,
      { role: 'assistant', content: gen.fullText },
      { role: 'user', content: responseBlock.trim() },
    ];
  }

  return {
    fullText: transcript,
    toolExecutions,
    weightsMatched,
  };
}
