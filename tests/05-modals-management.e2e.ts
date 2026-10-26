import { test } from '@e2e-dev/web';
import { expect } from 'e2e';

test('PHP Configuration Manager, Quick App Scaffolder, and Package Hub modals work end-to-end', async ({ app, screen }) => {
  await app.open('/');

  // ==========================================
  // 1. PHP Configuration Manager Modal
  // ==========================================
  await screen.getByRole('button', 'PHP Settings').tap();
  await expect(screen.getByRole('heading', 'PHP Configuration Manager')).toBeVisible();

  // Extensions Tab: verify extensions list
  await expect(screen.getByText('mysqli')).toBeVisible();
  await expect(screen.getByText('xdebug')).toBeVisible();

  // Switch to Directives Tab
  await screen.getByRole('button', 'Directives & Limits').tap();
  await expect(screen.getByText('memory_limit')).toBeVisible();

  // Edit memory_limit and save
  const memInput = screen.getByDisplayValue('256M');
  await expect(memInput).toBeVisible();
  await memInput.fill('512M');
  await screen.getByRole('button', 'Save').first().tap();
  await expect(screen.getByText('Directive memory_limit updated to 512M')).toBeVisible();

  // Close PHP Modal via Escape on the edited input
  await screen.getByDisplayValue('512M').press('Escape');
  await expect(screen.getByRole('heading', 'PHP Configuration Manager')).toBeHidden();

  // ==========================================
  // 2. Quick App Scaffolder Modal
  // ==========================================
  await screen.getByRole('button', 'Quick App').first().tap();
  await expect(screen.getByRole('heading', 'Quick App Scaffolder')).toBeVisible();

  // Select WordPress template
  const wpTemplateBtn = screen.getByRole('button', /WordPress/);
  await expect(wpTemplateBtn).toBeVisible();
  await wpTemplateBtn.tap();

  // Enter project name
  const nameInput = screen.getByPlaceholder('e.g. my-awesome-app');
  await expect(nameInput).toBeVisible();
  await nameInput.fill('my-super-blog');

  // Submit form
  await screen.getByRole('button', 'Provision Project').tap();

  // Verify toast and new virtual host in table
  await expect(screen.getByText('Project created successfully!')).toBeVisible();
  await expect(screen.getByRole('link', 'my-super-blog.test')).toBeVisible();

  // ==========================================
  // 3. Modular Package Hub Modal
  // ==========================================
  await screen.getByRole('button', 'Packages').tap();
  await expect(screen.getByRole('heading', 'Modular Package Hub')).toBeVisible();

  // Verify packages
  await expect(screen.getByText('PHP 8.3 (Latest)')).toBeVisible();
  await expect(screen.getByText('PHP 8.2 (LTS)')).toBeVisible();

  // Install PHP 8.2
  const installBtn = screen.getByRole('button', 'Install').first();
  await expect(installBtn).toBeVisible();
  await installBtn.tap();

  // Verify package state updates to Ready
  await expect(screen.getByText('Ready').nth(1)).toBeVisible();

  // Close Package Hub modal
  await screen.getByRole('button', 'Close').tap();
  await expect(screen.getByRole('heading', 'Modular Package Hub')).toBeHidden();
});
