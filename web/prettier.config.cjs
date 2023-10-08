module.exports = {
	"printWidth": 80,
	"plugins": [
		"prettier-plugin-svelte"
	],
	"pluginSearchDirs": [
		"."
	],
	"overrides": [
		{
			"files": "*.svelte",
			"options": {
				"parser": "svelte"
			}
		}
	]
}
