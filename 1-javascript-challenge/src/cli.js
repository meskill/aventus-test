const assert = require("node:assert").strict;
const { hash, randomRange } = require("./utils.js");
const { createLeaf } = require("./mock.js");
const { createMerkleTree, createProof, verifyProof } = require("./tree.js");

exports.main = async function () {
  const numberOfLeaves = parseInt(process.argv[2], 10);

  if (Number.isNaN(numberOfLeaves)) {
    throw new Error("Please provide number of leaves to create");
  }

  assert(numberOfLeaves > 0, "Number of leaves should be more than 0");

  const leaves = [];
  for (let i = 0; i < numberOfLeaves; i++) {
    let diceRoll = randomRange(0, 100);
    leaves.push(createLeaf(diceRoll));
  }

  const leaves_hashes = leaves.map((leaf) => hash(leaf.print()));
  let tree = createMerkleTree(leaves_hashes);
  console.log("Tree", tree);

  // This gets a random value
  const rv = randomRange(0, leaves.length);
  let proof = createProof(tree, rv);

  console.log("Random Leaf Index", rv);
  console.log("Proof", proof);
  // Now we verify
  assert(verifyProof(tree, leaves_hashes[rv], proof));
};
