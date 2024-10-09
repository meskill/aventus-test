const { createHash } = require("crypto");
const fs = require("fs");

let currentIndex = 0;

function createLeaf(seed) {
  return {
    account: "Account" + seed,
    token: someMagicalHexValue(seed),
    balance: seed * 100,
    print: function () {
      return `${this.account}-${this.token}:${this.balance}`;
    },
  };
}
function createMerkleTree(leaves) {
  let tree = {
    nodes: [leaves],
    root: undefined,
  };
  const maxDepth = 10;
  for (var i = 0; i < maxDepth; i++) {
    let newNodes = createMerkleTreeLevel(leaves);
    tree.nodes.push(newNodes);
    if (newNodes.length == 1) {
      tree.root = newNodes[0];
      return tree;
    } else if (newNodes.length == 0) {
      return tree;
    } else {
      leaves = newNodes;
    }
  }
}
async function main() {
  let leaves = [];
  for (
    let leavesCreated = 0;
    leavesCreated < process.argv[2];
    leavesCreated++
  ) {
    let diceRoll = Math.trunc(Math.random() * 100);
    leaves.push(createLeaf(diceRoll));
  }
  // This gets a random value
  let rv = Math.trunc(Math.random() * leaves.length);
  leaves = leaves.map((leaf) => hash(leaf.print()));
  let tree = createMerkleTree(leaves);
  console.log("Root", tree.root, "\nTree", tree.nodes);
  let proof = createProof(tree, rv);
  console.log("Leaf Index", rv);
  console.log("Proof", proof);
  // Now we verify
  leaf = leaves[rv];
  for (const neighbour of proof) {
    let preHash;
    if (leaf < neighbour) {
      preHash = leaf + neighbour;
    } else {
      preHash = neighbour + leaf;
    }
    leaf = hash(preHash);
  }
  //Check validity
  console.log(leaf == tree.root);
  function createProof(tree, leafIndex) {
    const resultObj = {
      merklePath: [],
      treeWithoutRoot: "",
    };
    currentIndex = leafIndex;
    resultObj.treeWithoutRoot = remLast(tree.nodes);
    for (const nodesOfLevel of resultObj.treeWithoutRoot) {
      let pairIndex = findIndex(currentIndex);
      if (nodesOfLevel[pairIndex]) {
        resultObj.merklePath.push(nodesOfLevel[pairIndex]);
      } else {
        resultObj.merklePath.push("");
      }
      currentIndex = reduceIndexForNextLevel(pairIndex);
    }
    return resultObj.merklePath;
  }
  function findIndex() {
    if (currentIndex % 2 == 0) return currentIndex + 1;
    else return currentIndex - 1;
  }
}
function createMerkleTreeLevel(leaves) {
  const numLeaves = leaves.length;
  if (numLeaves < 2) {
    return numLeaves == 1 ? [hash(leaves[0])] : [];
  }
  let treeNodes = [];
  let a = "";
  let b = a;
  for (let i = 0; i < leaves.length; i++) {
    let indexIsEvenCheck = i % 2;
    if (indexIsEvenCheck == 0) a = leaves[i];
    else {
      b = leaves[i];
      let preHash;
      if (a < b) {
        preHash = a + b;
      } else {
        preHash = b + a;
      }
      treeNodes.push(hash(preHash));
      a = "";
      b = "";
    }
  }
  if (1 <= a.length) {
    treeNodes.push(hash(a));
  }
  return treeNodes;
}
function reduceIndexForNextLevel(index) {
  return Math.trunc(index / 2);
}
function someMagicalHexValue(seed) {
  const seedQuarter = seed / 4;
  let shiftedSeedQuarter = 1 << seedQuarter;
  const someVal = shiftedSeedQuarter * seed;
  return someVal.toString(16);
}
function remLast(list) {
  let result = [];
  for (i = 0; i < list.length - 1; i++) {
    result.push(list[i]);
  }
  return result;
}
function checkTreeSize(tree) {
  const MAX_TREE = 1000000;
  if (tree.length > MAX_TREE) {
    console.log("Max tree size reached");
  }
  return null;
}
function hash(a) {
  let hash = createHash("sha256");
  let updatedHash = hash.update(a);
  let updatedHashAsHex = updatedHash.digest("hex");
  return updatedHashAsHex;
}
(async () => {
  await main();
})().catch((e) => {
  console.log(e);
  process.exit(0);
});
