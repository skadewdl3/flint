const { sum, multiply } = require("./utils");

// This will trigger the no-console ESLint warning
console.log("Welcome to JS Sample Project!");

function main() {
  const num1 = 5;
  const num2 = 10;

  const sumResult = sum(num1, num2);
  const multiplyResult = multiply(num1, num2);

  return {
    sumResult,
    multiplyResult,
  };
}

// This will trigger the semi ESLint error (missing semicolon)
const result = main();

module.exports = { main };
