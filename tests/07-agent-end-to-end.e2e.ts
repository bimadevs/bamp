import { test } from '@e2e-dev/web';
import { expect } from 'e2e';

test('AI agent validates and controls BAMP stack end-to-end', { timeout: 300_000 }, async ({ app, screen, agent }) => {
  await app.open('/');

  // 1. Initial screen assertion by AI model
  await expect(screen.getByRole('heading', 'BAMP')).toBeVisible();
  await agent.assert('the BAMP dashboard shows core services and virtual hosts');

  // 2. AI model starts all services
  await agent.act('start all services by tapping the Start All button');
  await expect(screen.getByText('Stack Status: Running')).toBeVisible();
  await agent.assert('the stack status is Running');
});
