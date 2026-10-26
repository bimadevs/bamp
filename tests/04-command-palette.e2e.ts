import { test } from '@e2e-dev/web';
import { expect } from 'e2e';

test('command palette (Ctrl+K) search and quick action execution', async ({ app, screen }) => {
  await app.open('/');

  // 1. Open Command Palette via Search button
  const searchBtn = screen.getByRole('button', 'Search Ctrl+K');
  await expect(searchBtn).toBeVisible();
  await searchBtn.tap();

  // 2. Verify Command Palette is visible
  const searchInput = screen.getByPlaceholder('Search virtual hosts or actions...');
  await expect(searchInput).toBeVisible();
  await expect(screen.getByText('Quick Actions')).toBeVisible();

  // 3. Search for switch action
  await searchInput.fill('Switch');
  const switchAction = screen.getByRole('button', /Switch to Apache Web Server/);
  await expect(switchAction).toBeVisible();

  // Execute quick action
  await switchAction.tap();

  // Verify palette closed and action took effect
  await expect(screen.getByPlaceholder('Search virtual hosts or actions...')).toBeHidden();
  await expect(screen.getByText('Active Web Server: APACHE')).toBeVisible();

  // 4. Open Command Palette again
  await searchBtn.tap();
  const searchInput2 = screen.getByPlaceholder('Search virtual hosts or actions...');
  await expect(searchInput2).toBeVisible();

  // Search virtual hosts
  await searchInput2.fill('wordpress');
  await expect(screen.getByRole('button', /wordpress\.test/)).toBeVisible();

  // 5. Close with Escape key
  await searchInput2.press('Escape');
  await expect(screen.getByPlaceholder('Search virtual hosts or actions...')).toBeHidden();
});
