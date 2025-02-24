import stylisticJs from '@stylistic/eslint-plugin-js';
import stylistic from '@stylistic/eslint-plugin';

export default [
  {
    "plugins": {
      "@stylistic/js": "stylisticJs",
      "@stylistic": "stylistic"
    },
    "rules": {
      "arrow-spacing": {},
      "quotes": {
        "@stylistic/quotes": [
          "error",
          "double",
          {
            "allowTemplateLiterals": true,
            "avoidEscape": true
          }
        ],
        "quotes": [
          "error",
          "double",
          {
            "allowTemplateLiterals": true,
            "avoidEscape": true
          }
        ]
      },
      "key-spacing": {},
      "semi": {
        "@stylistic/semi": [
          "error",
          true
        ],
        "semi": [
          "error",
          true
        ]
      },
      "indent": {
        "indent": [
          "error",
          4,
          {
            "SwitchCase": 5,
            "CallExpression": {},
            "FunctionDeclaration": {
              "parameters": 3,
              "body": 2
            },
            "FunctionExpression": {}
          }
        ],
        "@stylistic/indent": [
          "error",
          4,
          {
            "SwitchCase": 5,
            "CallExpression": {},
            "FunctionDeclaration": {
              "parameters": 3,
              "body": 2
            },
            "FunctionExpression": {}
          }
        ]
      },
      "linebreak-style": {},
      "object-curly-spacing": {},
      "space-before-function-paren": {},
      "comma-dangle": {},
      "no-trailing-spaces": {
        "@stylistic/no-trailing-spaces": [
          "error",
          false
        ],
        "no-trailing-spaces": [
          "error",
          false
        ]
      }
    }
  }
];