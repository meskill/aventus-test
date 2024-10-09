"use strict";

const assert = require("node:assert").strict;
const { pairHash } = require("./utils.js");

// Some reasonable limit for size of the tree
const MAX_TREE = 1000000;

exports.createMerkleTree = (leaves) => {
  assert(leaves.length < MAX_TREE, "Max tree size reached");

  let nodes = leaves;
  let tree = [leaves];

  while (true) {
    if (nodes.length == 1) {
      break;
    }

    let newNodes = createMerkleTreeLevel(nodes);

    tree.push(newNodes);
    nodes = newNodes;
  }

  return tree.reverse();
};

exports.createProof = (tree, leafIndex) => {
  const merklePath = [];
  let nodeIndex = leafIndex;

  for (let i = tree.length - 1; i > 0; i--) {
    let pairIndex = findSiblingIndex(nodeIndex);

    if (pairIndex >= tree[i].length) {
      // if number of nodes is odd reuse the last one as sibling to itself
      pairIndex--;
    }
    merklePath.push(tree[i][pairIndex]);

    nodeIndex = reduceIndexForNextLevel(pairIndex);
  }

  return merklePath;
};

exports.verifyProof = (tree, leaf_hash, proof) => {
  for (const neighbour of proof) {
    leaf_hash = pairHash(leaf_hash, neighbour);
  }

  return leaf_hash == tree[0][0];
};

function findSiblingIndex(index) {
  return index % 2 == 0 ? index + 1 : index - 1;
}

function createMerkleTreeLevel(nodes) {
  let parentNodes = [];

  for (let i = 1; i < nodes.length; i += 2) {
    parentNodes.push(pairHash(nodes[i - 1], nodes[i]));
  }

  if (nodes.length % 2 == 1) {
    let lastNode = nodes[nodes.length - 1];
    // if number of nodes is odd reuse the last one as sibling to itself
    parentNodes.push(pairHash(lastNode, lastNode));
  }
  return parentNodes;
}

function reduceIndexForNextLevel(index) {
  return Math.trunc(index / 2);
}
