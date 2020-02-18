module.exports = {
  displayName: "Unit Tests",
  testRegex: "(/.*.(test|spec))\\.(ts?|js?)$",
  transform: {
    "^.+\\.ts?$": "ts-jest"
  },
  moduleFileExtensions: ["ts", "js", "json"],
  setupFilesAfterEnv: [
    "./jest.setup.js"
  ]
};
