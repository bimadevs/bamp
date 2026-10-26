import { test } from '@e2e-dev/web';
import { expect } from 'e2e';

test('live activity and process logs drawer expansion, filtering, and clear', async ({ app, screen }) => {
  await app.open('/');

  // 1. Initial State: Logs drawer header is visible
  const drawerHeader = screen.getByText(/Live Activity & Process Logs/);
  await expect(drawerHeader).toBeVisible();

  // Filter buttons should not be visible initially when drawer is collapsed
  await expect(screen.getByRole('button', 'ALL')).toBeHidden();

  // 2. Click to open logs drawer
  await drawerHeader.tap();

  // Filter buttons are now visible
  await expect(screen.getByRole('button', 'ALL')).toBeVisible();
  await expect(screen.getByRole('button', 'NGINX')).toBeVisible();
  await expect(screen.getByRole('button', 'PHP')).toBeVisible();
  await expect(screen.getByRole('button', 'SUPERVISOR')).toBeVisible();
  await expect(screen.getByRole('button', 'Clear')).toBeVisible();

  // 3. Filter by SUPERVISOR
  await screen.getByRole('button', 'SUPERVISOR').tap();

  // 4. Clear logs
  await screen.getByRole('button', 'Clear').tap();
  await expect(screen.getByText('No log messages yet.')).toBeVisible();

  // 5. Close logs drawer
  await drawerHeader.tap();
  await expect(screen.getByRole('button', 'ALL')).toBeHidden();
});
