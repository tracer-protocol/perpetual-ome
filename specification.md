# Tracer Perpetual Swaps OME Technical Specification #

## Document Metadata ##

**Version**: 0.4.2

**Authors**: Jack McPherson \<[jackm@lionsmane.dev](mailto:jackm@lionsmane.dev)\>

## Preface ##
### Pseudocode ###
This document makes use of pseudocode written in a Rust-like language. While the intent is that such code follows Rust's syntax and semantics, it may deviate from this at times.

Additionally, while all JSON blocks in this document are valid JSON, they have been formatted to be human-readable. Actual implementations should likely minimise these.

### Boilerplate ###
Boilerplate such as accessers (i.e., getters and setters), constructors, destructors, etc. have been omitted from this specification. This has been done for numerous reasons:

 - To keep the specification simple and brief
 - To avoid unnecessarily constraining compliant implementations
 - Ease of maintenance of the specification itself

## Introduction ##
The Order Matching Engine (OME) is an architectural component of the entire Tracer Perpetual Swaps protocol. It has two responsibilities:

 - Match user-submitted orders
 - Maintain an order book from both user-submitted order flow and upstream order state

The OME's inputs are user-submitted orders and it's outputs are pairings of orders that have successfully matched. These 2-tuples are then submitted upstream to the [Executioner](https://github.com/tracer-protocol/executioner).

## Rationale ##
While the OME is not necessary for the correct operation of the Tracer Perpetual Swaps protocol itself, it provides important usability and efficiency gains to the network overall.

The OME exposes a simple and familiar RESTful JSON API to both frontend developers and market makers. In essence, it abstracts away the complexity associated with blockchain interaction.

Additionally, the OME is much faster than all modern blockchain implementations, as it is simply a traditional program being executed on classical computing infrastructure.

## Data Structures ##
### `OrderSide` ###
#### Description ####

The `OrderSide` type represents which side of the market an order is on.

#### Fields ####

N/A

#### Domain ####
The `OrderSide` type is an enumerated type with the following fields:

 - Bid
 - Ask

### `Order` ###
#### Description ####

The `Order` type represents an order in a Tracer Perpetual Swaps market.

#### Fields ####

| Name | Type | Description |
| ---- | ---- | ----------- |
| ID | 32-byte Keccak-256 digest | The unique identifier of the order |
| Address | Ethereum address | The Ethereum address of the trader submitting this order |
| Market | Ethereum address | The Ethereum address of the Tracer Perpetual Swaps market |
| Side | `OrderSide` | The side of the market for the order |
| Price | 256-bit unsigned integer | The price of the order |
| Amount | 256-bit unsigned integer | The quantity of the order |
| Expiration | Timestamp | The time at which the order will cease being valid |
| signedData | 65-byte-long EIP-712 signature | An EIP-712 compliant digital signature for the order |

#### Domain ####

N/A

### `Book` ###
#### Description ####

The `Book` type is an ADT representing the entire state of a given Tracer Perpetual Swaps
market. It is essentially a container type holding orders.

#### Fields ####

| Name | Type | Description |
| ---- | ---- | ----------- |
| Market | Ethereum address | The Etheruem address of the Tracer Perpetual Swaps smart contract for the market |
| Bids | Mapping from prices to collections of orders | The bid side of the market |
| Asks | Mapping from prices to collections of orders | The ask side of the market |
| LTP | 256-bit unsigned integer | The last traded price of the market |
| Depth | Pair of 256-bit unsigned integers | The depth of each side of the order book (i.e., bid then ask) |
| Crossed | Boolean | Whether the book is currently crossed or not |
| Spread | 256-bit signed integer | The current spread of the book |

#### Domain ####

N/A

## Internal Application Programming Interfaces #
### `Book` ###

The `Book` type has the following internal API defined on it:

#### `submit` ####

```rust
pub fn submit(&mut self, order: &mut Order) -> Result<(), BookError>
```

##### Description #####
The `submit` function is used to submit an order to the book. As such, it acts as the first entry point for a (**valid**) order within the OME.

If the order can be matched, the matching engine will match it and submit the matched pair to the Exectioner. Otherwise, `submit` will add the order to the book accordingly.

Note that `submit` necessarily **mutates** the order book state.

##### Parameters #####

| Name | Type | Description |
| ---- | ---- | ----------- |
| Order | `Order` | The order being submitted to the matching engine |

##### Return Type #####

The `submit` function's return type is such that it is able to return an appropriate error type in the event of an (implementation-defined) error condition.

#### `cancel` ####

```rust
pub fn cancel(&mut self, order: OrderId) -> Result<(), BookError>
```

##### Description #####

The `cancel` function is used to cancel an existing order currently stored within the order book.

Note that `cancel` necessarily **mutates** the order book state.

##### Parameters #####

| Name | Type | Description |
| ---- | ---- | ----------- |
| ID | 32-byte Keccak-256 digest | The unique identifier of the order to be cancelled |

##### Return Type #####

On success, `cancel` returns the timestamp representing when the order was cancelled. Otherwise, it returns an (implementation-defined) error condition.

### External Application Programming Interfaces ###

For both the Submission and Execution APIs, the following rules apply to all routes:

 - If the request payload is malformed in any way, the server must return a HTTP 400 Bad Request
 - In the event of a miscellaneous error (i.e., an error condition not covered explicitly by this specification), the server must return a HTTP 500 Internal Server Error

#### Submission API ####

The Submission API is the user-facing interface of the OME. It accepts order flow as input and returns various information as output. The Submission API implements JSON-REST.

| Object | Create | Read | Update | Destroy | Index |
| ------ | ------ | ---- | ------ | ------- | ----- |
| Order  | `POST /book/{market}/order` | `GET /book/{market}/order/{order_id}` | N/A | `DELETE /book/{market}/order/{order_id}` | `GET /book/{market}/order` |
| Book   | `POST /book` | `GET /book/{market}` | N/A | N/A | `GET /book` |

##### `GET book/` #####

###### Description ######

HTTP GET requests to the `book/` endpoint must return the entire list of Tracer Perpetual Swaps markets that the OME knows about.

###### Request ######

N/A

###### Response ######

An example response payload is:

```json
{
    "markets": [
        "0xfb59B91646cd0890F3E5343384FEb746989B66C7",
        "0x88efAbd098E18C575a6699FaA04c8d6F4050f040",
        "0xeE40e733c4e478947D7c112C1B11c2918E1F2942"
    ]
}
```

##### `POST book/` #####

###### Description ######

HTTP POST requests to the `book/` endpoint must create a new order book associated with the specified market address, unless the market already exists in the OME.

###### Request ######

| Name | Type | Description |
| ---- | ---- | ----------- |
| Market | String | The Ethereum address of the market |

An example request payload is:

```json
{
    "market": ""0xeE40e733c4e478947D7c112C1B11c2918E1F2942
}
```

###### Response ######

On success:

```json
{
    "status": 200,
    "message": "Market created"
}
```

On failure, the appropriate status code and error message. For example,

```json
{
    "status": 409,
    "message": "Market already exists"
}
```

| Error Condition | HTTP Status Code |
| --------------- | ---------------- |
| Specified market already exists | 409 Conflict |

##### `GET book/{market}` #####

###### Description ######

HTTP GET requests to the `book/{market}` endpoint display the order book for that market.

###### Request ######

N/A

###### Response ######

An example response payload is:

```json
{
    "market": "0xe66cf41c0ca141f78d33785c2aef9b7f359d8f79",
    "bids": {
            "300000000000000000000": [
                {
                    "id": "0xb970ea16a754e6f4f31e0ffc13aef75b86bd84df0bddd6a197dc91d35eafb40a",
                    "user": "0xeaf2b0b940f2cb3aeb85cc1fe5e758856ab5530a",
                    "target_tracer": "0xe66cf41c0ca141f78d33785c2aef9b7f359d8f79",
                    "side": "Ask",
                    "price": "300000000000000000000",
                    "amount": "120000000000000000000",
                    "amount_left": "120000000000000000000",
                    "expiration": "1624322757",
                    "created": "1623977157",
                    "signed_data": "0xdc7ae45111271ec2855c62311f8835bb4db24ae37c746fd2ac539308752463ec0cb5456d9e1121a485fa9ff59a2c7543b6ab6e1ab456a6dd4d61af30ee7c94361b"
                },
                {
                    "id": "0xf6c83e3641a08ec21aebc01296ff12f5a46780f0fbadb1c8101309123b95d2c6",
                    "user": "0x000000cd089424309a429e070b981c792cae2a0f",
                    "target_tracer": "0xe66cf41c0ca141f78d33785c2aef9b7f359d8f79",
                    "side": "Ask",
                    "price": "300000000000000000000",
                    "amount": "330000000000000000000",
                    "amount_left": "330000000000000000000",
                    "expiration": "1624325757",
                    "created": "1623977009",
                    "signed_data": "0xdc7ae45111271ec2855c62311f8835bb4db24ae37c746fd2ac539308752463ec0cb5456d9e1121a485fa9ff59a2c7543b6ab6e1ab456a6dd4d61af30ee7c94361b"
                }
            ]
    },
    "asks": {
        "340000000000000000000": [
                {
                    "id": "0xff223d4641a08ec21aebc01296ab12f5a46780f0fbadb1c8101309123b95d2c6",
                    "user": "0x00ab12cd089424309a429e070b981c788cae2aff",
                    "target_tracer": "0xe66cf41c0ca141f78d33785c2aef9b7f359d8f79",
                    "side": "Ask",
                    "price": "340000000000000000000",
                    "amount": "90000000000000000000",
                    "amount_left": "90000000000000000000",
                    "expiration": "1724325757",
                    "created": "1523977009",
                    "signed_data": "0xdc7ae45111271ec2855c62311f8835bb4db24ae37c746fd2ac539308752463ec0cb5456d9e1121a485fa9ff59a2c7543b6ab6e1ab456a6dd4d61af30ee7c94361b"
                }
        ]
    },
    "LTP": "320000000000000000000",
    "depth": [
        2,
        1
    ],
    "crossed": false,
    "spread": "40000000000000000000"
}
```

| Error Condition | HTTP Status Code |
| --------------- | ---------------- |
| Market doesn't exist | 404 Not Found |

##### `GET order/{order_id}` #####

###### Request ######

N/A

###### Response ######

An example response payload is:

```json
{
    "id": "0xb970ea16a754e6f4f31e0ffc13aef75b86bd84df0bddd6a197dc91d35eafb40a",
    "user": "0xeaf2b0b940f2cb3aeb85cc1fe5e758856ab5530a",
    "target_tracer": "0xe66cf41c0ca141f78d33785c2aef9b7f359d8f79",
    "side": "Ask",
    "price": "300000000000000000000",
    "amount": "120000000000000000000",
    "amount_left": "120000000000000000000",
    "expiration": "1624322757",
    "created": "1623977157",
    "signed_data": "0xdc7ae45111271ec2855c62311f8835bb4db24ae37c746fd2ac539308752463ec0cb5456d9e1121a485fa9ff59a2c7543b6ab6e1ab456a6dd4d61af30ee7c94361b"
}
```

##### `DELETE order/{order_id}` #####

###### Request ######

N/A

###### Response ######

```json
{
    "status": 200,
    "message": "Order cancelled",
}
```

| Error Condition | HTTP Status Code |
| --------------- | ---------------- |
| Order doesn't exist | 404 Not Found |

##### `POST book/{market}/order` #####

###### Request ######

An example request payload is:

```json
{
    "address": "0xdeadbeef",
    "side": "Ask",
    "price": 4380090000,
    "amount": 4,
    "expiration": 1595997399,
    "signedData": "0xcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadff"
}
```

###### Response ######

The `message` field of the response JSON object will be one of three strings:

 - `"Add"` (the order was added to the order book without crossing)
 - `"PartialMatch"` (the order was partially matched and the remainder was added to the order book)
 - `"FullMatch"` (the order was fully matched with another order on the order book already)

```json
{
    "status": 200,
    "message": "Add",
}
```


| Error Condition | HTTP Status Code |
| --------------- | ---------------- |
| Market doesn't exist | 404 Not Found |

