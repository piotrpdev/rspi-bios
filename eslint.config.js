import globals from "globals";
import pluginJs from "@eslint/js";
import jsInHtml from "eslint-plugin-html";
import html from "@html-eslint/eslint-plugin";
import eslintPluginPrettierRecommended from "eslint-plugin-prettier/recommended";

export default [
    { languageOptions: { globals: globals.browser } },
    pluginJs.configs.recommended,
    eslintPluginPrettierRecommended,
    {
        ...html.configs["flat/recommended"],
        files: ["templates/**/*.html"],
        plugins: {
            ...html.configs["flat/recommended"].plugins,
            ...eslintPluginPrettierRecommended.plugins,
            jsInHtml,
        },
        // languageOptions: { sourceType: "script" },
        rules: {
            ...html.configs["flat/recommended"].rules, // Must be defined. If not, all recommended rules will be lost
            ...eslintPluginPrettierRecommended.rules,
            "@html-eslint/indent": ["error", 4],
            "@html-eslint/no-extra-spacing-attrs": [
                "error",
                { enforceBeforeSelfClose: true },
            ],
            "@html-eslint/require-closing-tags": [
                "error",
                { selfClosing: "always" },
            ],
            "@html-eslint/attrs-newline": "off",
        },
    },
];
