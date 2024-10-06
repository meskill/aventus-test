# EXERCISE 3 - Substrate challenge

## Description

The goal of this exercise is to write a Substrate pallet that is acting as a price feed (oracle) to
an off-chain system.

Imagine a software that needs to know the price of a particular product (eg: gold) in order to
run some business logic. This software has an API to accept the price from an external
source (a Substrate pallet in this case). The API is called updatePrice and takes a number
as an input among other things.

Your task is to create a pallet that will read the price of some product (it can be anything)
from a source and post it to the updatePrice API of the software described above.

The frequency of update is up to you to decide, but bear in mind of the following when
designing your solution:
- Not all validators of the chain are trusted
- Include enough information in the post data to allow the API to validate it (its enough
to discuss how the API would validate, you don’t need to write the API validation)
- Finality (Bonus point)

## How to run

- for nix users: run `nix develop` inside the directory
- otherwise make sure [rustup](https://rustup.rs) is installed. Additional tools like `clang`, `protobuf` etc could be also required, for reference check the [getting started script](https://github.com/paritytech/polkadot-sdk/blob/9128dca3c544a5327f0a89241aea1409d67c81b0/scripts/getting-started.sh) from polkadot-sdk.

### Run interactively

1. Start the mock api `cargo run -p external-api` that will run simple backend server and show any post data it accepts. It will act as our "software" that relies on blockchain data
2. Start node in dev mode with `cargo run -p minimal-template-node -- --dev`. That will start the blockchain with additional functionality for the development.
3. Navigate to [polkadot.js.org](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#) with your browser
4. On the web page make sure you're connected to the dev node.
5. Go to `Developer -> Sudo`
6. Find the palette with name starting with `oraclePriceFeed` and it's extrinsic `setPrice`
7. Enter some value and press `Submit Sudo`
8. After some time you'll find the state of blockchain is update, new event `SendPrice` is generated and in the terminal for mock api you should see the post message content sent by palette.
9. Feel free to experiment with different values, try another instance of the palette and see how the output differs in that case

### Run tests

From the repo's root run command `cargo test`.

## Implementation

The code is based on the [minimal template](https://github.com/paritytech/polkadot-sdk-minimal-template). The palette implementation is located inside `pallets/oracle-price-feed` folder where the source and test code could be found. Some of the changed were added to the `runtime` to setup the developed palette.

Palette is implemented as off-chain worker that for every block checks if the value for price has changed on-chain and sends the updated value to configured API. Palette is configurable with the ability to change API endpoint and specify multiple instances of palette to track multiple values.

The palette sends additional data along the price that could be used to verify it's validity against the on-chain state. Block and storage references that are sent could be used in the validation process by utilizing additional Substrate functionality like JSON-RPC.

## Possible improvements

Palette sends the data on every block creation without waiting for its finalization. By using only finalized blocks we can be sure that data won't be changed on-chain. According to the [issue](https://github.com/paritytech/substrate/issues/6742) implementation of such functionality would require to use JSON-RPC calls from the off-chain worker.
