# Spice
Single-sided liquidity protocol
## Description 
The protocol uses the pool delta to maintain the initial liquidity in the pool. If the delta is negative, the commission is increased, if the delta is positive, the base pool commission is applied
```
delta = current_liquidity - initial_liquidity
```
Providers contribute liquidity to pools with a single token and profit in the form of pool fee. Protocol revenue is deposited in pools to increase liquidity and cover possible losses of providers
