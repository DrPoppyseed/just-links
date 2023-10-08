module.exports = {
	root: true,
	parser: "@typescript-eslint/parser",
	extends: [
		"eslint:recommended",
		'plugin:svelte/recommended',
		"plugin:@typescript-eslint/recommended",
		"prettier"
	],
	plugins: [
		"@typescript-eslint"
	],
	parserOptions: {
		sourceType: "module",
		ecmaVersion: 2020,
		project: './tsconfig.json',
		extraFileExtensions: ['.svelte']
	},
	overrides: [
		{
			files: ['*.svelte'],
			parser: 'svelte-eslint-parser',
			parserOptions: {
				parser: '@typescript-eslint/parser'
			}
		}
	],
	env: {
		browser: true,
		node: true
	}
}
