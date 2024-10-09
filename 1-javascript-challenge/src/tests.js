const test = require("node:test");
const assert = require("node:assert").strict;
const { createMerkleTree, createProof, verifyProof } = require("./tree.js");
const { createLeaf } = require("./mock.js");
const { hash } = require("./utils.js");

const createTestLeaves = (numberOfLeaves) => {
  const leaves = [];
  for (let i = 0; i < numberOfLeaves; i++) {
    leaves.push(createLeaf(i));
  }

  return leaves;
};

test("test merkle tree size 1 deep check", (t) => {
  const leaves = createTestLeaves(1);
  const leaves_hashes = leaves.map((leaf) => hash(leaf.print()));
  let tree = createMerkleTree(leaves_hashes);
  let proof = createProof(tree, 0);

  assert.deepEqual(tree, [
    ["fab9601fcc9f9a365bbc0a2a3bfdaab1c5063e81d06cc7136a11eca3c9338aed"],
  ]);
  assert.deepEqual(proof, []);

  assert(verifyProof(tree, leaves_hashes[0], proof));
});

test("test merkle tree size 2 deep check", (t) => {
  const leaves = createTestLeaves(2);
  const leaves_hashes = leaves.map((leaf) => hash(leaf.print()));
  let tree = createMerkleTree(leaves_hashes);

  let proof = createProof(tree, 0);

  assert.deepEqual(tree, [
    ["86112757b4d669685e14c0a806ea0116f287028e26a387637e14c3707e5a619b"],
    [
      "fab9601fcc9f9a365bbc0a2a3bfdaab1c5063e81d06cc7136a11eca3c9338aed",
      "8d107469907af8b3afd357e9d92f9603bdd3537ff75c27d5e0ff438fca5440ce",
    ],
  ]);
  assert.deepEqual(proof, [
    "8d107469907af8b3afd357e9d92f9603bdd3537ff75c27d5e0ff438fca5440ce",
  ]);

  assert(verifyProof(tree, leaves_hashes[0], proof));

  proof = createProof(tree, 1);

  assert.deepEqual(proof, [
    "fab9601fcc9f9a365bbc0a2a3bfdaab1c5063e81d06cc7136a11eca3c9338aed",
  ]);

  assert(verifyProof(tree, leaves_hashes[1], proof));
});

test("test merkle tree size 5 deep check", (t) => {
  const leaves = createTestLeaves(5);
  const leaves_hashes = leaves.map((leaf) => hash(leaf.print()));
  let tree = createMerkleTree(leaves_hashes);

  assert.deepEqual(tree, [
    ["f412c9366f1ef595b7f3c17ec74a4f6193480dc43887d521671dbfd48f575bb0"],
    [
      "4f2dc50731af729d29524339b9ec99cb4240770140e2f5a0c129ea1a2414555d",
      "e6f018586b2179605c13ca842cca35b528ebab6f450e5842e84e4741e0658dd3",
    ],
    [
      "86112757b4d669685e14c0a806ea0116f287028e26a387637e14c3707e5a619b",
      "7074782382b7d240ac263611e3729cb767400ce9cd5e0a3bb5a8f627b44e05a2",
      "6dd99c4c1cbb83ff773ed97dc05048384d91870b66be4ad8fdc98747e0c65d0c",
    ],
    [
      "fab9601fcc9f9a365bbc0a2a3bfdaab1c5063e81d06cc7136a11eca3c9338aed",
      "8d107469907af8b3afd357e9d92f9603bdd3537ff75c27d5e0ff438fca5440ce",
      "9d4f9d35013ebd37a8c93a230d626eb5e17d4fa989b3390b81778fc5636f871c",
      "012e1810263715806eb6e15e7426d3270dcd08f78db90be8d86e5998c44d14fc",
      "985e69f982f73cd54e1bae72893909daf57dc29e80fdddc856de0027bd074a1f",
    ],
  ]);

  let proof = createProof(tree, 0);

  assert.deepEqual(proof, [
    "8d107469907af8b3afd357e9d92f9603bdd3537ff75c27d5e0ff438fca5440ce",
    "7074782382b7d240ac263611e3729cb767400ce9cd5e0a3bb5a8f627b44e05a2",
    "e6f018586b2179605c13ca842cca35b528ebab6f450e5842e84e4741e0658dd3",
  ]);

  assert(verifyProof(tree, leaves_hashes[0], proof));

  proof = createProof(tree, 3);

  assert.deepEqual(proof, [
    "9d4f9d35013ebd37a8c93a230d626eb5e17d4fa989b3390b81778fc5636f871c",
    "86112757b4d669685e14c0a806ea0116f287028e26a387637e14c3707e5a619b",
    "e6f018586b2179605c13ca842cca35b528ebab6f450e5842e84e4741e0658dd3",
  ]);

  assert(verifyProof(tree, leaves_hashes[3], proof));

  proof = createProof(tree, 4);

  assert.deepEqual(proof, [
    "985e69f982f73cd54e1bae72893909daf57dc29e80fdddc856de0027bd074a1f",
    "6dd99c4c1cbb83ff773ed97dc05048384d91870b66be4ad8fdc98747e0c65d0c",
    "4f2dc50731af729d29524339b9ec99cb4240770140e2f5a0c129ea1a2414555d",
  ]);

  assert(verifyProof(tree, leaves_hashes[4], proof));
});

test("test merkle tree size 20", (t) => {
  const leaves = createTestLeaves(20);
  const leaves_hashes = leaves.map((leaf) => hash(leaf.print()));
  let tree = createMerkleTree(leaves_hashes);

  for (let i = 0; i < leaves.length; i++) {
    let proof = createProof(tree, i);
    assert(verifyProof(tree, leaves_hashes[i], proof));
  }
});
