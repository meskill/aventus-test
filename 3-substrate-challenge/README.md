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
