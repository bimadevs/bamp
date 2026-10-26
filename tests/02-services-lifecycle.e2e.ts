import { test } from '@e2e-dev/web';
import { expect } from 'e2e';

test('services lifecycle (Start All, Stop All, and individual control) works seamlessly', async ({ app, screen }) => {
  await app.open('/');

  // 1. Initial State: Stack is stopped
  await expect(screen.getByText('Stack Status: Stopped')).toBeVisible();
  await expect(screen.getByText('0 of 5 services active')).toBeVisible();

  // 2. Global "Start All"
  const startAllBtn = screen.getByRole('button', 'Start All');
  await expect(startAllBtn).toBeVisible();
  await startAllBtn.tap();

  // Verify status updates to Running
  await expect(screen.getByText('All active services started.')).toBeVisible();
  await expect(screen.getByText('Stack Status: Running')).toBeVisible();
  await expect(screen.getByText('4 of 5 services active')).toBeVisible();

  // Verify active services
  await expect(screen.getByText('Nginx Web Server')).toBeVisible();
  await expect(screen.getByText('Apache HTTP Server')).toBeVisible();
  await expect(screen.getByText('Standby')).toBeVisible();

  // 3. Global "Stop All"
  const stopAllBtn = screen.getByRole('button', 'Stop All');
  await expect(stopAllBtn).toBeVisible();
  await stopAllBtn.tap();

  // Verify status updates to Stopped
  await expect(screen.getByText('All services stopped.')).toBeVisible();
  await expect(screen.getByText('Stack Status: Stopped')).toBeVisible();
  await expect(screen.getByText('0 of 5 services active')).toBeVisible();

  // 4. Individual service start: Start Mailpit
  const startButtons = await screen.getByRole('button', 'Start').all();
  expect(startButtons.length).toBeGreaterThan(0);

  // Click the last start button (Mailpit)
  await startButtons[startButtons.length - 1].tap();

  // Verify Mailpit starts and stack status becomes running
  await expect(screen.getByText('Stack Status: Running')).toBeVisible();
  await expect(screen.getByText('1 of 5 services active')).toBeVisible();

  // Verify Stop and Restart buttons now exist
  await expect(screen.getByRole('button', 'Stop').first()).toBeVisible();
  await expect(screen.getByRole('button', 'Restart').first()).toBeVisible();

  // Tap Restart
  await screen.getByRole('button', 'Restart').first().tap();
  await expect(screen.getByText('Stack Status: Running')).toBeVisible();

  // Tap Stop
  await screen.getByRole('button', 'Stop').first().tap();
  await expect(screen.getByText('Stack Status: Stopped')).toBeVisible();
  await expect(screen.getByText('0 of 5 services active')).toBeVisible();
});
