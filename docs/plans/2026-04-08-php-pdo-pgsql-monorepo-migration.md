# PHP PDO PostgreSQL Connector - Monorepo Migration and Release Setup

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate PHP PDO PostgreSQL connector from staging repo to aurora-dsql-connectors monorepo and establish automated release pipeline to Packagist

**Architecture:** The PHP connector will follow the same monorepo pattern as other connectors (Python, Ruby, Node.js, etc.). Code lives in `php/pdo_pgsql/`, releases are triggered by git tags (`php/pdo_pgsql/v*`), and packages are published to Packagist. The mirror repo (`aurora-dsql-php-pdo-pgsql`) will receive subtree splits for users who prefer a standalone repository.

**Tech Stack:** PHP 8.2+, Composer, PHPUnit, GitHub Actions, Packagist

---

## Task 1: Analyze and Prepare Source Code

**Files:**
- Read: `/Volumes/workplace/php/src/AxdbJumpstartStaging/php/pdo_pgsql/`
- Read: `/Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql/`

**Step 1: Compare source and destination**

Run:
```bash
cd /Volumes/workplace/php/src/AxdbJumpstartStaging/php/pdo_pgsql
find . -type f -not -path "*/vendor/*" -not -path "*/.git/*" | wc -l
```

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql
find . -type f | wc -l
```

Expected: Source has ~30+ files, destination has ~2 placeholder files

**Step 2: Verify tests run in source**

Run:
```bash
cd /Volumes/workplace/php/src/AxdbJumpstartStaging/php/pdo_pgsql
composer install --no-interaction
vendor/bin/phpunit --testsuite unit
```

Expected: All unit tests pass (integration tests may require DSQL cluster)

**Step 3: Review composer.json compatibility**

Read `/Volumes/workplace/php/src/AxdbJumpstartStaging/php/pdo_pgsql/composer.json`

Verify:
- Package name is `aws/aurora-dsql-pdo-pgsql`
- Homepage points to monorepo path: `github.com/awslabs/aurora-dsql-connectors/tree/main/php/pdo_pgsql`
- Dependencies are compatible with latest versions

---

## Task 2: Backup Existing Monorepo PHP Directory

**Files:**
- Read: `/Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql/`
- Create: `/Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql.backup/`

**Step 1: Check git status**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git status php/pdo_pgsql/
```

Expected: Clean working directory or tracked changes

**Step 2: Create backup of current placeholder**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
cp -r php/pdo_pgsql php/pdo_pgsql.backup
git log --oneline php/pdo_pgsql/ | head -5
```

Expected: Backup created, git history visible

**Step 3: Document current state**

Create:
```bash
echo "Backup created $(date)" > php/pdo_pgsql.backup/BACKUP_INFO.txt
echo "Files replaced during migration" >> php/pdo_pgsql.backup/BACKUP_INFO.txt
ls -la php/pdo_pgsql/ >> php/pdo_pgsql.backup/BACKUP_INFO.txt
```

---

## Task 3: Copy Source Code to Monorepo

**Files:**
- Copy from: `/Volumes/workplace/php/src/AxdbJumpstartStaging/php/pdo_pgsql/`
- Copy to: `/Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql/`

**Step 1: Remove placeholder files**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql
rm -rf src/ composer.json
ls -la
```

Expected: Only backup directory remains (if created)

**Step 2: Copy source files (excluding vendor and git)**

Run:
```bash
rsync -av --exclude='vendor/' --exclude='.git/' --exclude='.phpunit.result.cache' --exclude='composer.lock' \
  /Volumes/workplace/php/src/AxdbJumpstartStaging/php/pdo_pgsql/ \
  /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql/
```

Expected: All source files copied

**Step 3: Verify directory structure**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql
find . -type d -maxdepth 2 | sort
find . -type f -name "*.php" | head -10
```

Expected output structure:
```
./example
./src
./test
./src/AuroraDsql.php
./src/DsqlConfig.php
./src/DsqlPdo.php
./test/unit/
./test/integration/
```

**Step 4: Test composer install**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql
composer install --no-interaction
```

Expected: Dependencies installed successfully

---

## Task 4: Create PHP CI Workflow

**Files:**
- Create: `/Volumes/workplace/aurora-dsql-connectors/.github/workflows/php-pdo-pgsql-ci.yml`

**Step 1: Write CI workflow file**

Create `.github/workflows/php-pdo-pgsql-ci.yml`:

```yaml
name: PHP PDO_PGSQL Connector CI

on:
  pull_request:
    paths:
      - "php/pdo_pgsql/**"
      - ".github/workflows/php-pdo-pgsql-ci.yml"
  push:
    branches:
      - main
    paths:
      - "php/pdo_pgsql/**"
      - ".github/workflows/php-pdo-pgsql-ci.yml"
  workflow_dispatch:

defaults:
  run:
    working-directory: php/pdo_pgsql

jobs:
  test:
    name: PHP ${{ matrix.php-version }} - ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        php-version: ["8.2", "8.3", "8.4"]
        os: [ubuntu-latest]
    
    steps:
      - name: Checkout code
        uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6

      - name: Setup PHP
        uses: shivammathur/setup-php@v2
        with:
          php-version: ${{ matrix.php-version }}
          extensions: pdo, pdo_pgsql
          coverage: none

      - name: Validate composer.json
        run: composer validate --strict

      - name: Install dependencies
        run: composer install --prefer-dist --no-progress --no-interaction

      - name: Run unit tests
        run: vendor/bin/phpunit --testsuite unit

      - name: Check code style
        run: composer check-style || echo "Style check not configured"
        continue-on-error: true

  integration-test:
    name: Integration Tests
    runs-on: ubuntu-latest
    if: github.event_name != 'pull_request' || contains(github.event.pull_request.labels.*.name, 'run-integration-tests')
    
    steps:
      - name: Checkout code
        uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6

      - name: Setup PHP
        uses: shivammathur/setup-php@v2
        with:
          php-version: "8.3"
          extensions: pdo, pdo_pgsql
          coverage: none

      - name: Install dependencies
        run: composer install --prefer-dist --no-progress --no-interaction

      - name: Create DSQL cluster
        uses: ./.github/workflows/dsql-cluster-create.yml
        with:
          cluster-name: php-pdo-pgsql-ci-${{ github.run_id }}

      - name: Run integration tests
        run: vendor/bin/phpunit --testsuite integration
        env:
          DSQL_ENDPOINT: ${{ steps.create-cluster.outputs.endpoint }}

      - name: Cleanup DSQL cluster
        if: always()
        uses: ./.github/workflows/dsql-cluster-delete.yml
        with:
          cluster-name: php-pdo-pgsql-ci-${{ github.run_id }}
```

**Step 2: Verify workflow syntax**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
cat .github/workflows/php-pdo-pgsql-ci.yml | head -20
```

Expected: Valid YAML with correct structure

---

## Task 5: Create PHP Release Workflow

**Files:**
- Create: `/Volumes/workplace/aurora-dsql-connectors/.github/workflows/php-pdo-pgsql-release.yml`

**Step 1: Write release workflow**

Create `.github/workflows/php-pdo-pgsql-release.yml`:

```yaml
name: Publish PHP PDO_PGSQL Connector to Packagist

permissions: {}

on:
  push:
    tags:
      - "php/pdo_pgsql/v*"

defaults:
  run:
    working-directory: php/pdo_pgsql

jobs:
  wait-for-ci:
    name: Wait for CI to pass
    runs-on: ubuntu-latest
    steps:
      - uses: lewagon/wait-on-check-action@a08fbe2b86f9336198f33be6ad9c16b96f92799c # v1.6.0
        with:
          ref: ${{ github.sha }}
          running-workflow-name: "Wait for CI to pass"
          repo-token: ${{ secrets.GITHUB_TOKEN }}
          wait-interval: 30
          check-regexp: "PHP 8\\."

  validate-tag:
    name: Validate release tag
    needs: wait-for-ci
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.extract.outputs.version }}
    steps:
      - name: Extract version from tag
        id: extract
        run: |
          VERSION="${GITHUB_REF_NAME#php/pdo_pgsql/v}"
          echo "version=$VERSION" >> "$GITHUB_OUTPUT"
          echo "Publishing version: $VERSION"

      - name: Checkout code
        uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6

      - name: Verify version in Version.php
        run: |
          VERSION="${{ steps.extract.outputs.version }}"
          if ! grep -q "VERSION = '$VERSION'" src/Version.php; then
            echo "Error: Version mismatch. src/Version.php does not contain VERSION = '$VERSION'"
            exit 1
          fi

  trigger-packagist:
    name: Trigger Packagist update
    needs: validate-tag
    runs-on: ubuntu-latest
    steps:
      - name: Notify Packagist
        run: |
          echo "Packagist auto-updates from GitHub releases"
          echo "Package will be available at: https://packagist.org/packages/aws/aurora-dsql-pdo-pgsql"
          echo "Version: ${{ needs.validate-tag.outputs.version }}"

  create-github-release:
    name: Create GitHub Release
    needs: validate-tag
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - name: Checkout code
        uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6

      - name: Extract changelog
        id: changelog
        run: |
          VERSION="${{ needs.validate-tag.outputs.version }}"
          CHANGELOG=$(sed -n "/## $VERSION/,/## [0-9]/p" CHANGELOG.md | sed '$d')
          echo "changelog<<EOF" >> "$GITHUB_OUTPUT"
          echo "$CHANGELOG" >> "$GITHUB_OUTPUT"
          echo "EOF" >> "$GITHUB_OUTPUT"

      - name: Create GitHub Release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref_name }}
          release_name: PHP PDO_PGSQL Connector v${{ needs.validate-tag.outputs.version }}
          body: ${{ steps.changelog.outputs.changelog }}
          draft: false
          prerelease: false

  sync-mirror-repo:
    name: Sync to mirror repository
    needs: [validate-tag, create-github-release]
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - name: Checkout monorepo
        uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6
        with:
          fetch-depth: 0

      - name: Setup git
        run: |
          git config --global user.name "github-actions[bot]"
          git config --global user.email "github-actions[bot]@users.noreply.github.com"

      - name: Extract subtree to mirror repo
        env:
          GITHUB_TOKEN: ${{ secrets.MIRROR_REPO_TOKEN }}
        run: |
          VERSION="${{ needs.validate-tag.outputs.version }}"
          
          # Split subtree
          git subtree split --prefix=php/pdo_pgsql -b php-pdo-pgsql-release
          
          # Clone mirror repo
          git clone https://x-access-token:${GITHUB_TOKEN}@github.com/awslabs/aurora-dsql-php-pdo-pgsql.git mirror-repo
          cd mirror-repo
          
          # Merge subtree
          git checkout main || git checkout -b main
          git pull ../.. php-pdo-pgsql-release
          
          # Tag and push
          git tag "v${VERSION}"
          git push origin main
          git push origin "v${VERSION}"

  update-changelog:
    needs: [validate-tag, sync-mirror-repo]
    permissions:
      contents: write
      pull-requests: write
    uses: ./.github/workflows/update-changelog.yml
    with:
      tag: ${{ github.ref_name }}
    secrets: inherit
```

**Step 2: Verify workflow file**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
cat .github/workflows/php-pdo-pgsql-release.yml | grep -A5 "jobs:"
```

Expected: Valid YAML structure

---

## Task 6: Update Version File

**Files:**
- Modify: `/Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql/src/Version.php`

**Step 1: Read current version file**

Read: `/Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql/src/Version.php`

**Step 2: Set initial version**

Update version constant to `0.1.0`:

```php
<?php

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql;

final class Version
{
    public const VERSION = '0.1.0';
}
```

**Step 3: Verify version**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql
grep "VERSION = " src/Version.php
```

Expected: `public const VERSION = '0.1.0';`

---

## Task 7: Update CHANGELOG

**Files:**
- Modify: `/Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql/CHANGELOG.md`

**Step 1: Write initial changelog**

Update `CHANGELOG.md`:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - TBD

### Added
- Initial release of Aurora DSQL PHP PDO_PGSQL connector
- Automatic IAM token generation via AWS SDK for PHP
- SSL enforcement with verify-full mode
- OCC retry with exponential backoff and jitter
- PSR-3 compatible logging
- Connection string parsing support
- Support for AWS profiles and custom credentials providers
- Flexible host configuration (full endpoint or cluster ID)
- Region auto-detection from endpoint hostname

[0.1.0]: https://github.com/awslabs/aurora-dsql-connectors/releases/tag/php/pdo_pgsql/v0.1.0
```

**Step 2: Verify changelog format**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql
head -20 CHANGELOG.md
```

Expected: Proper Keep a Changelog format

---

## Task 8: Add README Updates

**Files:**
- Modify: `/Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql/README.md`

**Step 1: Update installation section**

Verify README contains correct monorepo installation:

```markdown
## Installation

```bash
composer require aws/aurora-dsql-pdo-pgsql
```

Or add to your `composer.json`:

```json
{
    "require": {
        "aws/aurora-dsql-pdo-pgsql": "^0.1"
    }
}
```
```

**Step 2: Add badge placeholders**

Add to top of README:

```markdown
# Aurora DSQL PHP PDO_PGSQL Connector

[![Latest Version](https://img.shields.io/packagist/v/aws/aurora-dsql-pdo-pgsql)](https://packagist.org/packages/aws/aurora-dsql-pdo-pgsql)
[![PHP Version](https://img.shields.io/packagist/php-v/aws/aurora-dsql-pdo-pgsql)](https://packagist.org/packages/aws/aurora-dsql-pdo-pgsql)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
```

**Step 3: Verify README**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql
head -30 README.md
```

Expected: Badges and installation instructions visible

---

## Task 9: Update Root README

**Files:**
- Modify: `/Volumes/workplace/aurora-dsql-connectors/README.md`

**Step 1: Add PHP section to connector table**

Read current README to find insertion point after Ruby section

**Step 2: Insert PHP connector entry**

Add after Ruby section (around line 43):

```markdown
### PHP

| Package | Description | Packagist | License |
|---------|-------------|-----------|---------|
| [aurora-dsql-pdo-pgsql](./php/pdo_pgsql/) | PDO_PGSQL connector for Aurora DSQL | [![Packagist](https://img.shields.io/packagist/v/aws/aurora-dsql-pdo-pgsql)](https://packagist.org/packages/aws/aurora-dsql-pdo-pgsql) | ![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg) |
```

**Step 3: Update installation section**

Add PHP to installation examples (around line 76):

```markdown
# PHP
composer require aws/aurora-dsql-pdo-pgsql
```

**Step 4: Add documentation link**

Add to documentation section (around line 93):

```markdown
- [PHP PDO_PGSQL connector documentation](./php/pdo_pgsql/README.md)
```

**Step 5: Verify changes**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
grep -A2 "### PHP" README.md
grep "composer require aws/aurora" README.md
```

Expected: PHP connector visible in main README

---

## Task 10: Test CI Workflow Locally

**Files:**
- Run: `/Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql/`

**Step 1: Install dependencies**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql
composer install --no-interaction
```

Expected: Clean install

**Step 2: Run unit tests**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql
vendor/bin/phpunit --testsuite unit
```

Expected: All tests pass

**Step 3: Validate composer.json**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql
composer validate --strict
```

Expected: No errors or warnings

---

## Task 11: Commit Changes to Monorepo

**Files:**
- Commit: All new/modified files in `/Volumes/workplace/aurora-dsql-connectors/`

**Step 1: Check git status**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git status
```

Expected: Shows new PHP files and workflow files

**Step 2: Stage PHP connector files**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git add php/pdo_pgsql/
git add .github/workflows/php-pdo-pgsql-*.yml
git add README.md
```

**Step 3: Create commit**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git commit -m "$(cat <<'EOF'
feat(php): add PDO_PGSQL connector to monorepo

Migrates PHP PDO_PGSQL connector from staging repository to monorepo structure.

Added:
- Full PHP source code in php/pdo_pgsql/
- Unit and integration tests
- CI workflow for testing across PHP 8.2, 8.3, 8.4
- Release workflow with Packagist integration
- Mirror repo sync workflow
- Documentation and examples

The connector provides automatic IAM authentication, OCC retry with
exponential backoff, and PSR-3 logging for Aurora DSQL.
EOF
)"
```

Expected: Commit created successfully

**Step 4: Verify commit**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git log -1 --stat
```

Expected: Commit shows all added files

---

## Task 12: Setup Packagist Integration

**Files:**
- Manual: Packagist configuration

**Step 1: Document Packagist setup requirements**

Create: `php/pdo_pgsql/docs/RELEASE.md`:

```markdown
# Release Process

## Prerequisites

1. **Packagist Registration**
   - Package: `aws/aurora-dsql-pdo-pgsql`
   - Repository: `https://github.com/awslabs/aurora-dsql-connectors`
   - Auto-update: Enabled via GitHub webhook

2. **GitHub Secrets**
   - `MIRROR_REPO_TOKEN`: Personal access token with repo write access

## Release Steps

1. Update version in `src/Version.php`
2. Update `CHANGELOG.md` with release notes
3. Commit changes: `git commit -m "chore: bump version to X.Y.Z"`
4. Create and push tag: `git tag php/pdo_pgsql/vX.Y.Z && git push origin php/pdo_pgsql/vX.Y.Z`
5. GitHub Actions will:
   - Run CI tests
   - Create GitHub release
   - Notify Packagist (auto-updates via webhook)
   - Sync to mirror repo

## Manual Packagist Setup (First Time Only)

1. Go to https://packagist.org/packages/submit
2. Enter repository URL: `https://github.com/awslabs/aurora-dsql-connectors`
3. Enable auto-update via GitHub webhook
4. Configure to watch `php/pdo_pgsql/v*` tags
```

**Step 2: Verify documentation**

Run:
```bash
cat /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql/docs/RELEASE.md
```

Expected: Release documentation visible

---

## Task 13: Setup Mirror Repository

**Files:**
- Manual: GitHub repository configuration

**Step 1: Document mirror repo setup**

Create notes file:

```bash
echo "Mirror Repository Setup Instructions

Repository: https://github.com/awslabs/aurora-dsql-php-pdo-pgsql

Setup Steps:
1. Verify repository exists and is empty (or ready to be overwritten)
2. Add GitHub secret MIRROR_REPO_TOKEN to aurora-dsql-connectors repo
3. Test subtree split manually before first release:
   cd /Volumes/workplace/aurora-dsql-connectors
   git subtree split --prefix=php/pdo_pgsql -b test-split
   
4. First sync will happen during first release workflow

Mirror repo will receive:
- Only files from php/pdo_pgsql/ directory
- Tag format: v* (not php/pdo_pgsql/v*)
- All commits affecting php/pdo_pgsql/
" > /tmp/mirror-repo-setup.txt
```

**Step 2: Test subtree split locally**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git subtree split --prefix=php/pdo_pgsql -b test-php-split
git log test-php-split --oneline | head -5
```

Expected: Separate branch created with only PHP commits

**Step 3: Cleanup test branch**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git branch -D test-php-split
```

---

## Task 14: Create Test Release Branch

**Files:**
- Branch: `php-pdo-pgsql-release-test`

**Step 1: Create test branch**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git checkout -b php-pdo-pgsql-release-test
```

**Step 2: Push test branch**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git push origin php-pdo-pgsql-release-test
```

Expected: Branch pushed to remote

**Step 3: Verify CI runs**

Check GitHub Actions:
```bash
gh run list --branch php-pdo-pgsql-release-test --limit 5
```

Expected: CI workflow triggered and running

---

## Task 15: Create Pull Request

**Files:**
- PR: GitHub Pull Request

**Step 1: Create PR from test branch**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
gh pr create --title "feat(php): Add PDO_PGSQL connector to monorepo" --body "$(cat <<'EOF'
## Summary

Migrates the PHP PDO_PGSQL connector from the staging repository to the monorepo structure, following the established pattern used by other language connectors.

## Changes

- **Source Code**: Full PHP connector implementation in `php/pdo_pgsql/`
  - IAM authentication with AWS SDK for PHP
  - OCC retry with exponential backoff
  - PSR-3 logging support
  - Comprehensive unit and integration tests

- **CI/CD**: 
  - PHP CI workflow testing PHP 8.2, 8.3, 8.4
  - Release workflow with Packagist integration
  - Mirror repo sync via git subtree split

- **Documentation**:
  - Updated root README with PHP connector
  - Connector README with usage examples
  - Release process documentation

## Testing

- ✅ Unit tests pass locally
- ✅ Composer validation passes
- ⏳ CI workflow running on this PR
- ⏳ Integration tests (requires DSQL cluster)

## Release Plan

1. Merge this PR to main
2. Setup Packagist webhook integration
3. Configure mirror repo access token
4. Create first release: `php/pdo_pgsql/v0.1.0`

## Checklist

- [x] Code migrated from staging repo
- [x] Tests pass locally
- [x] CI workflow configured
- [x] Release workflow configured
- [x] Documentation updated
- [ ] Packagist integration tested
- [ ] Mirror repo sync tested
EOF
)"
```

Expected: PR created successfully

**Step 2: Get PR URL**

Run:
```bash
gh pr view --json url --jq .url
```

Expected: Returns PR URL

---

## Task 16: Manual Pre-Release Checklist

**Files:**
- Manual verification steps

**Step 1: Verify CI passes**

```bash
gh pr checks
```

Expected: All checks pass

**Step 2: Register package on Packagist**

Manual steps:
1. Go to https://packagist.org/packages/submit
2. Login with AWS/AWSlabs account
3. Submit package: `aws/aurora-dsql-pdo-pgsql`
4. Repository URL: `https://github.com/awslabs/aurora-dsql-connectors`
5. Enable auto-update webhook

**Step 3: Configure GitHub secrets**

Manual steps:
1. Create Personal Access Token with `repo` scope
2. Add secret `MIRROR_REPO_TOKEN` to aurora-dsql-connectors repository
3. Verify secret is accessible to workflows

**Step 4: Verify mirror repo is ready**

```bash
gh repo view awslabs/aurora-dsql-php-pdo-pgsql
```

Expected: Repository exists and is accessible

---

## Task 17: Merge and Create First Release

**Files:**
- Merge: Pull Request
- Tag: `php/pdo_pgsql/v0.1.0`

**Step 1: Merge PR**

Run:
```bash
gh pr merge --squash --delete-branch
```

Expected: PR merged to main

**Step 2: Pull latest main**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git checkout main
git pull origin main
```

**Step 3: Update release date in CHANGELOG**

Update `CHANGELOG.md`:
```markdown
## [0.1.0] - 2026-04-08
```

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
sed -i '' 's/## \[0.1.0\] - TBD/## [0.1.0] - 2026-04-08/' php/pdo_pgsql/CHANGELOG.md
git add php/pdo_pgsql/CHANGELOG.md
git commit -m "chore(php): update release date in CHANGELOG"
git push origin main
```

**Step 4: Create and push release tag**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git tag php/pdo_pgsql/v0.1.0
git push origin php/pdo_pgsql/v0.1.0
```

Expected: Tag pushed, release workflow triggered

**Step 5: Monitor release workflow**

Run:
```bash
gh run watch
```

Expected: Release workflow completes successfully

---

## Task 18: Verify Release

**Files:**
- Verify: Packagist, GitHub Release, Mirror Repo

**Step 1: Verify GitHub Release**

Run:
```bash
gh release view php/pdo_pgsql/v0.1.0
```

Expected: Release created with changelog

**Step 2: Verify Packagist**

Run:
```bash
curl -s https://packagist.org/packages/aws/aurora-dsql-pdo-pgsql.json | jq '.package.versions["0.1.0"]'
```

Expected: Version 0.1.0 visible on Packagist (may take a few minutes)

**Step 3: Verify mirror repo**

Run:
```bash
gh repo view awslabs/aurora-dsql-php-pdo-pgsql
git clone https://github.com/awslabs/aurora-dsql-php-pdo-pgsql.git /tmp/mirror-test
cd /tmp/mirror-test
git tag
ls -la
```

Expected:
- Tag `v0.1.0` exists (without `php/pdo_pgsql/` prefix)
- Files match monorepo `php/pdo_pgsql/` structure

**Step 4: Test installation**

Run:
```bash
mkdir -p /tmp/test-install
cd /tmp/test-install
composer init --no-interaction
composer require aws/aurora-dsql-pdo-pgsql:^0.1
composer show aws/aurora-dsql-pdo-pgsql
```

Expected: Package installs from Packagist

---

## Task 19: Update Documentation

**Files:**
- Create: `php/pdo_pgsql/docs/MAINTENANCE.md`

**Step 1: Create maintenance guide**

Create `php/pdo_pgsql/docs/MAINTENANCE.md`:

```markdown
# Maintenance Guide

## Release Process

See [RELEASE.md](./RELEASE.md) for detailed release instructions.

## Mirror Repository

The mirror repository (`aurora-dsql-php-pdo-pgsql`) is automatically synced via git subtree split during the release workflow.

- **Source**: `php/pdo_pgsql/` in monorepo
- **Mirror**: `awslabs/aurora-dsql-php-pdo-pgsql` (standalone repo)
- **Tag format**: `v*` in mirror (not `php/pdo_pgsql/v*`)

Users can use either:
1. Monorepo: `github.com/awslabs/aurora-dsql-connectors`
2. Mirror: `github.com/awslabs/aurora-dsql-php-pdo-pgsql` (composer default)

## CI/CD Workflows

### CI Workflow
- **File**: `.github/workflows/php-pdo-pgsql-ci.yml`
- **Triggers**: PR, push to main, manual
- **Runs**: Unit tests on PHP 8.2, 8.3, 8.4

### Release Workflow
- **File**: `.github/workflows/php-pdo-pgsql-release.yml`
- **Triggers**: Tag push `php/pdo_pgsql/v*`
- **Steps**:
  1. Wait for CI
  2. Validate version
  3. Create GitHub release
  4. Sync mirror repo
  5. Update changelog

## Troubleshooting

### Packagist not updating
- Verify webhook is configured at https://packagist.org/packages/aws/aurora-dsql-pdo-pgsql
- Manually trigger update if needed

### Mirror repo sync failed
- Check GitHub secret `MIRROR_REPO_TOKEN` is valid
- Verify token has `repo` scope
- Check workflow logs for subtree split errors

### CI failing
- Verify PHP version compatibility
- Check for missing dependencies
- Ensure tests are isolated and don't require DSQL cluster
```

**Step 2: Commit documentation**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git add php/pdo_pgsql/docs/
git commit -m "docs(php): add release and maintenance documentation"
git push origin main
```

---

## Task 20: Cleanup

**Files:**
- Remove: Backup files and test artifacts

**Step 1: Remove backup directory**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
rm -rf php/pdo_pgsql.backup
```

**Step 2: Remove test install directory**

Run:
```bash
rm -rf /tmp/test-install /tmp/mirror-test
```

**Step 3: Verify clean state**

Run:
```bash
cd /Volumes/workplace/aurora-dsql-connectors
git status
```

Expected: Clean working directory

---

## Post-Migration Tasks

### Ongoing Maintenance

1. **Deprecate staging repository**
   - Add deprecation notice to staging repo README
   - Point users to monorepo location
   - Archive staging repository after grace period

2. **Monitor first releases**
   - Watch for installation issues
   - Monitor GitHub issues
   - Update documentation based on feedback

3. **Establish release cadence**
   - Follow semantic versioning
   - Maintain CHANGELOG.md
   - Coordinate with other connectors

### Success Criteria

- ✅ PHP connector code in monorepo
- ✅ CI workflow running on PRs
- ✅ Release workflow tested
- ✅ Package published to Packagist
- ✅ Mirror repo receiving updates
- ✅ Documentation complete
- ✅ First release (v0.1.0) successful

---

## Notes

- **Manual steps required**: Packagist registration, GitHub secrets configuration
- **Mirror repo**: Requires `MIRROR_REPO_TOKEN` secret with repo write access
- **Integration tests**: May need DSQL cluster configuration for full test suite
- **Composer lock file**: Excluded from repo; generated during CI/CD
- **Version management**: Must update `src/Version.php` before each release

## Related Skills

- @superpowers:verification-before-completion - Run before claiming tasks complete
- @superpowers:systematic-debugging - Use if CI/CD issues arise
- @superpowers:requesting-code-review - Use before merging major changes
