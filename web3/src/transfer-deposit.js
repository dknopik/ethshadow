#!/usr/bin/env node

const yargs = require('yargs');
const fs = require('fs');
const Web3 = require('web3');

const argv = yargs
    .option('endpoint', {
        description: 'HTTP endpoint to which you want to use to transfer the deposit',
        type: 'string',
        demandOption: true,
        requiresArg: true,
    })
    .option('address-file', {
        description: 'Deposit contract address file',
        type: 'string',
        demandOption: true,
        requiresArg: true,
    })
    .option('contract-file', {
        description: 'Deposit contract file',
        type: 'string',
        demandOption: true,
        requiresArg: true,
    })
    .option('deposit-file', {
        description: 'Deposit data file generated by staking-deposit-cli',
        type: 'string',
        demandOption: true,
        requiresArg: true,
    })
    .help()
    .alias('help', 'h').argv;

(async function() {
    const web3 = new Web3(argv.endpoint);
    const accounts = await web3.eth.getAccounts();

    const json = JSON.parse(fs.readFileSync(argv.contractFile));
    const data = JSON.parse(fs.readFileSync(argv.depositFile));
    const address = fs.readFileSync(argv.addressFile).toString();

    const contract = new web3.eth.Contract(json.abi, address);

    const promises = data.map(async deposit => {
        const transaction = contract.methods.deposit(
            '0x' + deposit.pubkey,
            '0x' + deposit.withdrawal_credentials,
            '0x' + deposit.signature,
            '0x' + deposit.deposit_data_root,
        );
        const receipt = await transaction.send({
            from: accounts[0],
            // The amount from the file is in Gwei, so we need to transform it to wei
            value: deposit.amount * 1000000000,
            gas: 2000000,
            gasPrice: '147000000000',
        });
        process.stdout.write('.');
        return receipt;
    });

    await Promise.all(promises);
    process.exit();
})();