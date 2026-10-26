import { test } from '@e2e-dev/web';
import { expect } from 'e2e';

test('dashboard header and global tools work correctly', async ({ app, screen }) => {
  await app.open('/');

  // 1. Brand identity
  await expect(screen.getByRole('heading', 'BAMP')).toBeVisible();
  await expect(screen.getByText('Modern WAMP • Multi-PHP • Zero-Zombie')).toBeVisible();

  // 2. Active Web Server switcher (Nginx <-> Apache)
  const nginxBtn = screen.getByRole('button', 'Nginx');
  const apacheBtn = screen.getByRole('button', 'Apache');
  await expect(nginxBtn).toBeVisible();
  await expect(apacheBtn).toBeVisible();
  await expect(screen.getByText('Active Web Server: NGINX')).toBeVisible();

  // Switch to Apache
  await apacheBtn.tap();
  await expect(screen.getByText('Active Web Server: APACHE')).toBeVisible();
  await expect(screen.getByText('Switched active web server to APACHE')).toBeVisible();

  // Switch back to Nginx
  await nginxBtn.tap();
  await expect(screen.getByText('Active Web Server: NGINX')).toBeVisible();
  await expect(screen.getByText('Switched active web server to NGINX')).toBeVisible();

  // 3. Global tool buttons
  await expect(screen.getByRole('button', 'Terminal')).toBeVisible();
  await expect(screen.getByRole('button', 'PHP Settings')).toBeVisible();
  await expect(screen.getByRole('button', 'Packages')).toBeVisible();
  await expect(screen.getByRole('button', 'Install CA')).toBeVisible();
  await expect(screen.getByRole('button', 'Mailbox')).toBeVisible();
  await expect(screen.getByRole('button', 'Database')).toBeVisible();

  // 4. Trigger Root CA install and verify toast
  await screen.getByRole('button', 'Install CA').tap();
  await expect(screen.getByText('Root CA successfully installed in trust store.')).toBeVisible();

  // 5. Trigger Isolated Terminal and verify toast
  await screen.getByRole('button', 'Terminal').tap();
  await expect(screen.getByText('Terminal opened with BAMP environment variables.')).toBeVisible();
});
