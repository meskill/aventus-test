const { someMagicalHexValue } = require("./utils.js");

exports.createLeaf = (seed) => {
  return {
    account: "Account" + seed,
    token: someMagicalHexValue(seed),
    balance: seed * 100,
    print: function () {
      return `${this.account}-${this.token}:${this.balance}`;
    },
  };
};
