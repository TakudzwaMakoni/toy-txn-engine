# Toy transaction engine

## Assumptions

1. A referencing transaction (e.g. a dispute) made by client A on a *referenced*
 transaction made by client B (if thats valid) is performed on client B's account, 
 and not on client A's account.

2. Disputes are valid against deposits only. Others are ignored.

3. A frozen accounts means it cannot perform further deposits or withdrawals, any are ignored.

4. Balances and amounts are positive or zero.


## Deserialising the amount
Blockchains use big integers (such as u128) to represent balances.
for example, the Atom token may be denominated in micro-units (uatom),
so a value of 1,500,000 is 1.5 atom. 

I opted to implement a custom deserialisation of the decimal string, 
parsing them as u128 values for this toy model, where the last four digits 
are the decimals. Since integers are safer for simple maths operations, 
we can check for integer over/underflow. Its also easier to store in memory 
and we know their size.

This means that the maximum decimal representation that the app can
handle is `34028236692093846346337460743176821.1455`. 


# The Ledger

## streaming data
The model is designed to handle large csv files, so the app will
stream data from the file. Each line entry is deserialised into
a record, and each record is consumed after the transaction is processed.
This way even with a large csv file, we are memory efficient.
No need to load the entire dataset to memory.

## storing accounts
accounts are stored in a hashmap whose keys are the client id since
we will be accessing accounts many times while we stream the data.
this means that account lookup is O(1) time complexity and we will
process each transaction as fast as possible.

## storing transaction history
Again we want to be able to retrieve previous transactions as fast as possible
since we could be streaming a large file, so we are using a hashmap whose
key is the txn id. We only store deposits and withdrawals as the other 
transactions are references to these only. time complexity for lookup is O(1).

# error handling
Before processing the transactions the app will parse args for the file,
open the file, and create buffer reader for the csv data. 
If these preprocessing steps fail, this is considered a critical failure. 
The app will then immediately terminate with a standard error
before any processing, so as to not corrupt any client data.

After preprocessing, the app will go into the processing stage where any errors 
which occurred are wrapped in a custom `ProcessError`.

If a Record is malformed/corrupted, e.g. an unrecognised transaction type, 
or no amount is provided for a deposit/withdrawal, the filesream is no
longer reliable, so the app will abort processing and terminate 
with `ProcessError::ExternalErr`.

If a process error occurs within a single transaction e.g. attempting 
to withdraw with insufficient funds, the app should continue with the 
rest of the transactions. Expanded below:

### withdrawal errors
if an account tries to withdraw an amount greater than the available, the app
will fail the withdrawal with `ProcessError::ExternalErr`, but should 
proceed with other transactions.

### deposit errors
if an account tries to deposit an amount greater than the amount limit, 
which is 340282366920938463463374607431768211455 as u128, the app
will fail the deposit, but should proceed with other transactions.

### dispute, resolve and chargeback errors
If the reference transactions are not found in the ledgers transaction history, 
they are ignored, and the app will continue processing other transactions. 

