# 13 - Quick App Scaffolder & Package Hub

Status: resolved

## Goal
Implement interactive Quick App Scaffolder for Blank PHP, WordPress, and Laravel, and Package Hub GUI for modular PHP version installation.

## Requirements
1. Quick App Scaffolder:
   - Backend `scaffold_app`:
     - Blank PHP: creates project folder, index.php, MySQL database, vhost, opens in browser.
     - WordPress: downloads core, extracts, configures database and vhost.
     - Laravel: runs composer create-project, configures database and vhost.
   - Frontend modal with project type selector, project name input, and progress feedback.
2. Package Hub GUI:
   - Backend `get_packages` and `install_package`:
     - Lists available PHP versions (7.4, 8.1, 8.2, 8.3, 8.4) with status (installed / downloadable).
     - Installs package into `bin/php/<version>/`.
   - Frontend Package Hub modal or view.

## Completion Criteria
- [x] Quick App Scaffolder provisions projects and databases in seconds.
- [x] Package Hub displays installed and downloadable PHP versions.
- [x] Unit tests `test_scaffold_blank_app` and `test_package_hub_listing` pass.

## Comments
- Implemented `ToolsManager::scaffold_app` supporting Blank PHP, WordPress bootstrapping, and Laravel scaffolding.
- Implemented `ToolsManager::get_package_hub` and `ToolsManager::install_package`.
- Added interactive modals in `App.tsx` and Command Palette shortcuts.
