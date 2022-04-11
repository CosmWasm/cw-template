# Secret Contract Integration Tests

This is an example of how to write integration tests in Typescript using secret.js
The goal of integration tests is to test our contract on-chain, to test the integration between multiple transactions / queries and even to check the integration between multiple contracts.

In order to run the tests first you will need to install the dependencies by running the following command:
```sh
npm install
```

After you have all of the dependencies install you can run the following command in order to build and run your code:
```sh
npx ts-node [[ts_file_name.ts]]
```

You can also choose to debug your code by using the following steps (Using vscode):
1. Press `ctrl+shift+p`
2. Write `JavaScript Debug Terminal` and press `Enter`
3. In the new terminal you can run `npx ts-node [[ts_file_name.ts]]`
4. Your code will be running in debug mode and will stop on every breakpoint placed.

## Conventions

There are no strict conventions, the only recommandation is to write test functions with "snake_case" naming sense (Only for the function name)
It is very important for the code to be clear and verbose in its outputs also as for the test functions to be self explanatory.
