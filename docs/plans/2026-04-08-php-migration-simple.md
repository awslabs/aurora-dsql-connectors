# PHP PDO_PGSQL Connector - Simple Monorepo Migration

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Copy PHP connector from staging repo to monorepo and setup Packagist releases

**Architecture:** Lift and shift - copy code as-is, adapt workflow paths, setup release tag workflow

**Tech Stack:** PHP 8.2+, Composer, PHPUnit, GitHub Actions, Packagist

---

## Task 1: Copy Source Code

**Step 1: Remove placeholder files in monorepo**

```bash
cd /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql
rm -rf src/ composer.json
```

**Step 2: Copy all files from staging**

```bash
rsync -av --exclude='.git/' --exclude='vendor/' --exclude='.phpunit.result.cache' \
  /Volumes/workplace/php/src/AxdbJumpstartStaging/php/pdo_pgsql/ \
  /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql/
```

**Step 3: Verify files copied**

```bash
cd /Volumes/workplace/aurora-dsql-connectors/php/pdo_pgsql
ls -la
find . -name "*.php" -type f | head -10
```

Expected: All source files, tests, examples present

**Step 4: Commit**

```bash
cd /Volumes/workplace/aurora-dsql-connectors
git add php/pdo_pgsql/
git commit -m "feat(php): add PDO_PGSQL connector from staging repo"
```

---

## Task 2: Copy and Adapt CI Workflow

**Step 1: Copy CI workflow**

```bash
cp /Volumes/workplace/php/src/AxdbJumpstartStaging/.github/workflows/php-pdo-pgsql-ci.yml \
   /Volumes/workplace/aurora-dsql-connectors/.github/workflows/php-pdo-pgsql-ci.yml
```

**Step 2: Verify workflow (no changes needed - paths already correct)**

```bash
cat /Volumes/workplace/aurora-dsql-connectors/.github/workflows/php-pdo-pgsql-ci.yml | head -20
```

Expected: Workflow uses `php/pdo_pgsql/**` paths

**Step 3: Commit**

```bash
cd /Volumes/workplace/aurora-dsql-connectors
git add .github/workflows/php-pdo-pgsql-ci.yml
git commit -m "ci(php): add PHP PDO_PGSQL CI workflow"
```

---

## Task 3: Create Release Workflow

**Step 1: Create release workflow (copy from Python pattern)**

Create `.github/workflows/php-pdo-pgsql-release.yml`:

```yaml
name: Publish PHP PDO_PGSQL to Packagist

permissions: {}

on:
  push:
    tags:
      - "php/pdo_pgsql/v*"

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
          check-regexp: "PHP PDO_PGSQL"

  create-release:
    name: Create GitHub Release
    needs: wait-for-ci
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6

      - name: Extract version from tag
        id: version
        run: |
          VERSION="${GITHUB_REF_NAME#php/pdo_pgsql/v}"
          echo "version=$VERSION" >> "$GITHUB_OUTPUT"

      - name: Create Release
        run: |
          gh release create "${{ github.ref_name }}" \
            --title "PHP PDO_PGSQL v${{ steps.version.outputs.version }}" \
            --notes "Release ${{ steps.version.outputs.version }} - See [CHANGELOG](https://github.com/awslabs/aurora-dsql-connectors/blob/main/php/pdo_pgsql/CHANGELOG.md)" \
            --verify-tag
        env:
          GH_TOKEN: ${{ github.token }}

  update-changelog:
    needs: create-release
    permissions:
      contents: write
      pull-requests: write
    uses: ./.github/workflows/update-changelog.yml
    with:
      tag: ${{ github.ref_name }}
    secrets: inherit
```

**Step 2: Commit**

```bash
cd /Volumes/workplace/aurora-dsql-connectors
git add .github/workflows/php-pdo-pgsql-release.yml
git commit -m "ci(php): add Packagist release workflow"
```

---

## Task 4: Update Root README

**Step 1: Add PHP section to connector table**

Edit `README.md` - insert after Ruby section (around line 43):

```markdown
### PHP

| Package | Description | Packagist | License |
|---------|-------------|-----------|---------|
| [aurora-dsql-pdo-pgsql](./php/pdo_pgsql/) | PDO_PGSQL connector for Aurora DSQL | [![Packagist](https://img.shields.io/packagist/v/aws/aurora-dsql-pdo-pgsql)](https://packagist.org/packages/aws/aurora-dsql-pdo-pgsql) | ![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg) |
```

**Step 2: Add to installation section** (around line 78):

```markdown
# PHP
composer require aws/aurora-dsql-pdo-pgsql
```

**Step 3: Add to documentation section** (around line 94):

```markdown
- [PHP PDO_PGSQL connector documentation](./php/pdo_pgsql/README.md)
```

**Step 4: Commit**

```bash
cd /Volumes/workplace/aurora-dsql-connectors
git add README.md
git commit -m "docs: add PHP PDO_PGSQL connector to main README"
```

---

## Task 5: Create PR and Test

**Step 1: Push branch**

```bash
cd /Volumes/workplace/aurora-dsql-connectors
git checkout -b php-pdo-pgsql-migration
git push origin php-pdo-pgsql-migration
```

**Step 2: Create PR**

```bash
gh pr create --title "feat(php): Add PDO_PGSQL connector to monorepo" --body "
## Summary
Migrates PHP PDO_PGSQL connector from staging repo to monorepo.

## Changes
- Copy all source code, tests, examples from staging
- Add CI workflow (creates DSQL cluster, runs unit + integration tests)
- Add release workflow (tag -> GitHub release -> Packagist webhook)
- Update root README

## Testing
CI will run automatically on this PR.
"
```

**Step 3: Wait for CI to pass**

```bash
gh pr checks
```

**Step 4: Merge when ready**

```bash
gh pr merge --squash --delete-branch
```

---

## Task 6: Setup Packagist (One-Time Manual)

**Step 1: Register package on Packagist**

1. Go to https://packagist.org/packages/submit
2. Login with AWS account
3. Submit: `aws/aurora-dsql-pdo-pgsql`
4. Repository: `https://github.com/awslabs/aurora-dsql-connectors`
5. Enable auto-update webhook

**Note:** Packagist will automatically detect new tags matching `php/pdo_pgsql/v*`

---

## Task 7: Create First Release

**Step 1: Checkout main**

```bash
cd /Volumes/workplace/aurora-dsql-connectors
git checkout main
git pull origin main
```

**Step 2: Create and push tag**

```bash
git tag php/pdo_pgsql/v0.1.0
git push origin php/pdo_pgsql/v0.1.0
```

**Step 3: Watch release workflow**

```bash
gh run watch
```

Expected: 
- GitHub release created
- Packagist updates automatically (webhook)

**Step 4: Verify on Packagist**

```bash
curl -s https://packagist.org/packages/aws/aurora-dsql-pdo-pgsql.json | jq '.package.versions["0.1.0"]'
```

---

## Done!

**Total changes:**
- Copy `php/pdo_pgsql/` directory
- Add 2 workflow files
- Update root README
- Register on Packagist (one-time)

**Future releases:**
1. Make changes in `php/pdo_pgsql/`
2. Update `CHANGELOG.md`
3. Commit and push
4. Create tag: `git tag php/pdo_pgsql/vX.Y.Z && git push origin php/pdo_pgsql/vX.Y.Z`
5. GitHub Actions automatically creates release
6. Packagist automatically updates via webhook

No mirror repo needed - Packagist supports monorepo subdirectories!
