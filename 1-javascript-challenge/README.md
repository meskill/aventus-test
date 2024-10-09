# EXERCISE 1 - Javascript challenge

## Description

A Merkle Tree is a concept often used in Blockchains.

It is a binary tree where each leaf node represents the hash of some interesting data and each internal node is the hash of the concatenated contents of its two children. Merkle Trees often record groups of transactions, and the roots are published widely to serve as summaries of all recognised transactions on a given date.

By construction, the tree's root is a hash of all its leaves organised in a specific order. Since hash functions are hard to reverse, it is unfeasible to create a tree with a specific root if we don't know the inputs to it. We can prove a transaction happened before a certain date by showing it was a leaf of Merkle Tree that has already been published. Merkle Trees provide an efficient way to prove this inclusion in the tree. It is enough to show a path of neighbour-nodes from the leaf to the root. That is, a list that includes the sibling of the leaf node, then the sibling of its parent and so on until the root is reached.

The code in this file represents a node.js app that demonstrates how to create a tree, a proof of inclusion for a random leaf and then verifies that the proof is correct. The size of the tree to build is passed as a CLI argument.

Your goal in this exercise is two-fold:

1. Imagine you receive this code in a Github Pull-Request submitted by one of your team mates. Write a code review for it with comments as you see fit.
2. Improve the code if you are able. Ensure it builds and runs.
3. Run the test cases you deem necessary to convince yourself the code is working properly.
