// Optional: Custom pnpm configuration
// This file can be used to customize pnpm behavior if needed

function readPackage(pkg, context) {
  return pkg;
}

module.exports = {
  hooks: {
    readPackage
  }
};

