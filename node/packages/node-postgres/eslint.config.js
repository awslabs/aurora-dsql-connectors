/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import js from '@eslint/js';
import tseslint from '@typescript-eslint/eslint-plugin';
import tsparser from '@typescript-eslint/parser';
import header from 'eslint-plugin-header';
import importPlugin from 'eslint-plugin-import-x';

header.rules.header.meta.schema = false;

export default [
  js.configs.recommended,
  {
    files: ['src/**/*.ts', 'test/**/*.ts'],
    languageOptions: {
      parser: tsparser,
      parserOptions: {
        ecmaVersion: 2020,
        sourceType: 'module'
      },
      globals: {
        URL: 'readonly',
        process: 'readonly',
        console: 'readonly',
        Buffer: 'readonly'
      }
    },
    plugins: {
      '@typescript-eslint': tseslint,
      'header': header
    },
    rules: {
      'semi': ['error', 'always'],
      '@typescript-eslint/no-unused-vars': ['error', {
        'argsIgnorePattern': '^_',
        'varsIgnorePattern': '^_'
      }],
      '@typescript-eslint/no-explicit-any': 'warn',
      'no-unused-vars': 'off', // Use TypeScript version instead
      'no-redeclare': 'off', // Allow function overloads
      'no-undef': 'off', // TypeScript handles this
      'header/header': [
        'error',
        'block',
        [
          '',
          ' * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.',
          ' * SPDX-License-Identifier: Apache-2.0',
          ' '
        ]
      ]
    }
  },
  {
    files: ['src/**/*.ts'],
    plugins: {
      'import': importPlugin
    },
    rules: {
      'import/no-extraneous-dependencies': ['error', { devDependencies: false }]
    }
  },
  {
    files: ['test/**/*.ts'],
    plugins: {
      'import': importPlugin
    },
    rules: {
      '@typescript-eslint/no-explicit-any': 'off',
      'import/no-extraneous-dependencies': ['error', { devDependencies: true }]
    }
  }
];
