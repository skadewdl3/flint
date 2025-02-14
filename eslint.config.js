import stylisticJs from '@stylistic/eslint-plugin-js';
import stylistic from '@stylistic/eslint-plugin';

export default [
  {
    "plugins": {
      "@stylistic/js": "stylisticJs",
      "@stylistic": "stylistic"
    },
    "rules": {
      "indent": [
        "error",
        4,
        {
          "SwitchCase": 5,
          "FunctionDeclaration": {
            "parameters": 3,
            "body": 2
          }
        }
      ],
      "quotes": {
        "@stylistic/quotes": [
          "error",
          "double"
        ],
        "quotes": [
          "error",
          "double"
        ]
      },
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
      "max-len": {
        "max-len": [
          "error",
          120
        ],
        "@stylistic/max-len": [
          "error",
          120
        ]
      },
      "array-bracket-newline": {
        "array-bracket-newline": [
          "error",
          "always"
        ],
        "@stylistic/js/array-bracket-newline": [
          "error",
          "always"
        ]
      },
      "array-element-newline": {},
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