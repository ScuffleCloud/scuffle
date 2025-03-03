import js from '@eslint/js';
import eslintPluginSvelte from 'eslint-plugin-svelte';
import globals from 'globals';
import svelteParser from 'svelte-eslint-parser';
import tsEslint from 'typescript-eslint';
import eslintPluginPrettier from 'eslint-plugin-prettier/recommended';

export default tsEslint.config(
    js.configs.recommended,
    ...tsEslint.configs.recommended,
    ...eslintPluginSvelte.configs['flat/recommended'],
    eslintPluginPrettier,
    ...eslintPluginSvelte.configs['flat/prettier'],
    {
        languageOptions: {
            ecmaVersion: 2022,
            sourceType: 'module',
            globals: { ...globals.node, ...globals.browser },
            parser: svelteParser,
            parserOptions: {
                parser: tsEslint.parser,
                extraFileExtensions: ['.svelte'],
            },
        },
    },
    {
        ignores: [
            '**/.svelte-kit',
            '**/.vercel',
            '**/.yarn',
            '**/build',
            '**/node_modules',
            '**/package',
            '**/email-worker',
        ],
    },
);
