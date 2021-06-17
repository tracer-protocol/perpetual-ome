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
| ID | 256-bit unsigned integer | The unique identifier of the order |
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
pub fn cancel(&mut self, order: OrderId) -> Result<Option<DateTime<Utc>>, BookError>
```

##### Description #####

The `cancel` function is used to cancel an existing order currently stored within the order book.

Note that `cancel` necessarily **mutates** the order book state.

##### Parameters #####

| Name | Type | Description |
| ---- | ---- | ----------- |
| ID | 256-bit unsigned integer | The unique identifier of the order to be cancelled |

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
        "0xcafebeef",
        "0xdeadbeef",
        "0xcafecafe"
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
    "market": "0xcafebeef"
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
    "market": "0xcafebeef",
    "bids": {
        "8795": [
            {
                "id": 33,
                "address": "0xdeaddead",
                "side": "Bid",
                "price": 816000000000000,
                "amount": 2,
                "expiration": 1595997379
            }
        ]
    },
    "asks": [],
    "LTP": 45996,
    "depth": [
        1,
        0
    ],
    "crossed": false,
    "spread": -816000000000000
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
    "id": 12,
    "address": "0xdeadbeef",
    "side": "Ask",
    "price": 4380090000,
    "amount": 4,
    "expiration": 1595997399,
    "signedData": "0xcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadff"
}
```

##### `DELETE order/{order_id}` #####

###### Request ######

N/A

###### Response ######

An example response payload is:

```json
{
    "cancelled": 1591597771
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

An example response payload is:

```json
{
    "id": 88
}
```

| Error Condition | HTTP Status Code |
| --------------- | ---------------- |
| Market doesn't exist | 404 Not Found |

#### Execution API ####

The Execution API is the interface between the OME and the Executioner. It's a HTTP-JSON API, but is not RESTful.

Note that, unlike the Submission API, this API is **outward-facing** from the OME. That is, the OME pushes data to the Executioner and the Executioner simply provides responses back to the OME.

##### `POST /submit` #####

###### Request ######

| Parameter | Type | Description |
| --------- | ---- | ----------- |
| `makers` | `[Order]` | List of maker orders |
| `takers` | `[Order]` | List of taker orders |

An example request payload is:

```json
{
    "makers": [
        {
            "id": 12,
            "address": "0xdeadbeef",
            "side": "Ask",
            "price": 4380090000,
            "amount": 4,
            "expiration": 1595997399,
            "flags": [
                "flag2",
                "flag4"
            ],
            "signedData": "0xcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadcafebeefdeaddeadff"
        }
    ],
    "takers": [
        {
            "id": 144,
            "address": "0xdeadbeef",
            "side": "Bid",
            "price": 4380090000,
            "amount": 4,
            "expiration": 1595997900,
            "flags": [
                "flag2"
            ],
            "signedData": "0xcafebeefdeaddeadcafe0000deaddeadcafebeefdeadffffcafebeefdeaddeadcafebeefcafecafebeefcafedeaddeadcafebeefdeaddeadcafebeefdeaddeadff"
        }
    ]
}
```

###### Response ######

| Parameter | Type | Description |
| --------- | ---- | ----------- |
| `makers` | `[String]` | List of transaction hashes for maker orders |
| `takers` | `[String]` | List of transaction hashes for taker orders |


An example response payload is:

```json
{
    "makers": [
        "0xbdd3cf5004516b84f44df491e4ab857fc7d3b114bb1ce97f135f21bf1fada0cf",
        "0x9d89f925fe3317f6e2a76e3fb265ecb97c4352edfbad52e085473b0e4d9e363f",
        "0xb2aaa6c20a8e9d9a120ee0b90c064d331ada338d6a26192497da998d3d0794de"
    ],
    "takers": [
        "0x7374b7484e3ee62161b9a94401216981e38a65ee64117841d02dff6d1c7a3b3f",
        "0x0287be62efaa5667568b2fcba20d25fcefc26164c1164354863940852d4e65e3"
    ]
}
```

| Error Condition | HTTP Status Code |
| --------------- | ---------------- |
| Web3-related error occurred | 502 Bad Gateway |

