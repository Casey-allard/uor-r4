/**
 * The MCP tool whose input is one prompt.
 *
 * `prism_prompt` takes a `prism/prompt/1` prompt; its `app` member is the
 * input of prism-code's `prism_spec`. `SPEC_INPUT` below is that input schema,
 * copied from prism-code's MCP server (vendor/prism-code/src/journey/mcp.mjs,
 * which imports Node built-ins and so cannot load in a browser); the builder's
 * tests compare the two and fail on any difference.
 */
import { PROMPT_SCHEMA, inputSchema } from './prompt.mjs';

const CONDITIONS = {
  type: 'array',
  minItems: 1,
  description: 'All must hold. A choice field compares with is or is not and one of its choices; a number field compares with a whole number or another number field, as {"field": "income"}.',
  items: {
    type: 'object',
    properties: {
      field: { type: 'string' },
      op: { type: 'string', enum: ['is', 'is not', '<', '<=', '>', '>=', '=', '!='] },
      value: { anyOf: [{ type: 'string' }, { type: 'integer', minimum: 0 }, { type: 'object', properties: { field: { type: 'string' } }, required: ['field'] }] },
    },
    required: ['field', 'op', 'value'],
  },
};

const WHY = { type: 'string', description: 'Why, in a few words, such as "it enforces INV-001".' };

/** The prism/spec/1 shape, so an agent can draft a whole spec from one sentence. */
export const SPEC_INPUT = {
  type: 'object',
  description: 'A whole prism/spec/1 spec. Omit to read.',
  properties: {
    name: { type: 'string', description: 'Lower case and hyphens, such as loan-approval.' },
    title: { type: 'string' },
    purpose: { type: 'string', description: 'What the app decides, in one sentence.' },
    fields: {
      type: 'array',
      minItems: 1,
      maxItems: 8,
      description: 'One request is these values in order, separated by spaces.',
      items: {
        type: 'object',
        properties: {
          name: { type: 'string', description: 'A lower-case word.' },
          kind: { type: 'string', enum: ['number', 'choice'] },
          values: { type: 'array', items: { type: 'string' }, description: 'For a choice: two to 32 lower-case words.' },
          min: { type: 'integer', minimum: 0 },
          max: { type: 'integer', minimum: 0 },
          means: { type: 'string', description: 'What the field is, with its unit, such as "yearly income, in whole dollars".' },
        },
        required: ['name', 'kind'],
      },
    },
    outcomes: { type: 'array', minItems: 2, items: { type: 'string' }, description: 'Lower-case words; invalid is reserved.' },
    meanings: { type: 'object', additionalProperties: { type: 'string' }, description: 'What each answer means, such as {"review": "a person must decide"}.' },
    rules: { type: 'array', minItems: 1, maxItems: 24, description: 'In order; the first that matches decides.', items: { type: 'object', properties: { when: CONDITIONS, then: { type: 'string' }, why: WHY }, required: ['when', 'then'] } },
    otherwise: { type: 'string', description: 'The answer when no rule matches.' },
    otherwiseWhy: WHY,
    properties: { type: 'array', description: 'What must never or always happen; each becomes a theorem Lean proves.', items: { type: 'object', properties: { kind: { type: 'string', enum: ['never', 'always'] }, when: CONDITIONS, then: { type: 'string' }, why: { type: 'string', description: 'The harm it prevents, such as "poor credit history predicts default". Ask the user when you cannot tell.' } }, required: ['kind', 'when', 'then'] } },
    limits: { type: 'array', items: { type: 'string' }, description: 'What the app does not decide, and choices the rules make silently, such as "fair credit is treated as good".' },
    examples: { type: 'array', description: 'Requests and their expected answers; each becomes an acceptance vector.', items: { type: 'object', properties: { request: { type: 'string' }, response: { type: 'string' } }, required: ['request', 'response'] } },
  },
  required: ['purpose', 'fields', 'outcomes', 'rules', 'otherwise'],
};

export const TOOL = {
  name: 'prism_prompt',
  title: 'Build a whole Prism app from one prompt',
  description: `Write every file of a Prism application from one ${PROMPT_SCHEMA} prompt: the model the guest is built from and the theorems Lean proves about it (from app, a prism/spec/1 spec), and the architecture, design views, features and journeys, security and threat model, quality model and compliance obligations, each checked by the Lean kernel; plus the register rows, scenarios, tests, page description and OpenSpec spec of a repository created from UOR-Foundation/template. Push the files to such a repository and its gate and Pages workflow build, verify and publish the app.`,
  inputSchema: inputSchema(SPEC_INPUT),
};
