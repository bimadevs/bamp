import type { E2EConfig } from 'e2e';
import { web } from '@e2e-dev/web';
import { createOpenAICompatible } from '@ai-sdk/openai-compatible';

export default {
  tests: 'tests/**/*.e2e.ts',
  timeout: 300_000,
  targets: [
    {
      name: 'chromium',
      engine: web({
        browser: 'chromium',
        initScripts: [{ path: 'tests/fixtures/tauri-mock.js' }],
      }),
      app: {
        url: 'http://127.0.0.1:1420',
        command: { executable: 'pnpm', args: ['dev', '--host', '127.0.0.1', '--port', '1420'], log: '.e2e/logs/app.log', reuseExisting: true },
      },
    },
  ],
  agents: {
    default: {
      model: createOpenAICompatible({
        name: 'openai-compatible',
        baseURL: 'http://localhost:20128/v1',
        fetch: async (input, init) => {
          const response = await fetch(input, init);
          const text = await response.text();
          const clean = text.replace(/data:\s*\[DONE\]\s*$/, '').trim();
          return new Response(clean, {
            status: response.status,
            statusText: response.statusText,
            headers: response.headers,
          });
        },
      }).chatModel('kc/recursive/flint-v1.0:free'),
      system: 'You are a thorough QA agent. Verify every outcome on screen.',
    },
  },
} satisfies E2EConfig;
