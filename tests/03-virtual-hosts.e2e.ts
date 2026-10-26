import { test } from '@e2e-dev/web';
import { expect } from 'e2e';

test('virtual hosts table, PHP version switcher, and linked folders work correctly', async ({ app, screen }) => {
  await app.open('/');

  // 1. Verify Virtual Hosts section and initial table contents
  await expect(screen.getByText(/Virtual Hosts & Projects/)).toBeVisible();
  await expect(screen.getByRole('link', 'wordpress.test')).toBeVisible();
  await expect(screen.getByRole('link', 'laravel.test')).toBeVisible();

  // 2. Verify SSL badges (HTTPS vs HTTP)
  await expect(screen.getByText('HTTPS')).toBeVisible();
  await expect(screen.getByText('HTTP')).toBeVisible();

  // 3. Change PHP version for a virtual host
  const phpSelect = screen.getByRole('combobox').first();
  await expect(phpSelect).toBeVisible();
  await phpSelect.selectOption({ value: 'php83' });

  // Verify toast confirmation
  await expect(screen.getByText(/PHP version for wordpress\.test set to php83/)).toBeVisible();

  // 4. Link a custom project folder
  const linkFolderBtn = screen.getByRole('button', 'Link Folder');
  await expect(linkFolderBtn).toBeVisible();
  await linkFolderBtn.tap();

  // Verify toast and new virtual host in table
  await expect(screen.getByText('Linked project folder saved and virtual host generated.')).toBeVisible();
  await expect(screen.getByRole('link', 'my-linked-store.test')).toBeVisible();

  // 5. Refresh sites button
  const refreshBtn = screen.getByRole('button', 'Refresh sites');
  await expect(refreshBtn).toBeVisible();
  await refreshBtn.tap();
  await expect(screen.getByRole('link', 'my-linked-store.test')).toBeVisible();
});
