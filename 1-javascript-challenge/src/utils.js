const assert = require("node:assert").strict;
const { createHash } = require("crypto");

exports.hash = (a) => createHash("sha256").update(a).digest("hex");

exports.pairHash = (a, b) => {
  let sum;
  if (a < b) {
    sum = a + b;
  } else {
    sum = b + a;
  }

  return exports.hash(sum);
};

exports.someMagicalHexValue = (seed) => {
  const seedQuarter = seed / 4;
  let shiftedSeedQuarter = 1 << seedQuarter;
  const someVal = shiftedSeedQuarter * seed;
  return someVal.toString(16);
};

exports.randomRange = (a, b) => {
  assert(b > a);
  return a + Math.trunc(Math.random() * (b - a));
};
