# Penumbra App

## General Structure

The general structure of commands and responses is as follows:

### Action plan supported
 - Spend
 - Output
 - Ics20 Withdrawal
 - Delegate
 - Undelegate
 - Undelegate Claim
 - Delegator Vote
 - Position Open
 - Position Close
 - Position Withdraw
 - Action Dutch Auction Schedule
 - Action Dutch Auction End
 - Action Dutch Auction Withdraw

### Commands

| Field   | Type     | Content                | Note |
| :------ | :------- | :--------------------- | ---- |
| CLA     | byte (1) | Application Identifier | `0x80` |
| INS     | byte (1) | Instruction ID         |      |
| P1      | byte (1) | Parameter 1            |      |
| P2      | byte (1) | Parameter 2            |      |
| L       | byte (1) | Bytes in payload       |      |
| PAYLOAD | byte (L) | Payload                |      |

### Response

| Field   | Type     | Content     | Note                     |
| ------- | -------- | ----------- | ------------------------ |
| ANSWER  | byte (?) | Answer      | Depends on the command   |
| SW1-SW2 | byte (2) | Return code | See list of return codes |

## Return Codes

| Return code | Description             |
| ----------- | ----------------------- |
| `0x6400`    | Execution Error         |
| `0x6700`    | Wrong buffer length     |
| `0x6982`    | Empty buffer            |
| `0x6983`    | Output buffer too small |
| `0x6984`    | Data is invalid         |
| `0x6986`    | Command not allowed     |
| `0x6987`    | Tx is not initialized   |
| `0x6B00`    | P1/P2 are invalid       |
| `0x6D00`    | INS not supported       |
| `0x6E00`    | CLA not supported       |
| `0x6F00`    | Unknown                 |
| `0x6F01`    | Sign / verify error     |
| `0x9000`    | Success                 |

## Command Definitions

### GET_DEVICE_INFO

#### Command

| Field | Type     | Content                | Expected |
| ----- | -------- | ---------------------- | -------- |
| CLA   | byte (1) | Application Identifier | `0xE0`   |
| INS   | byte (1) | Instruction ID         | `0x01`   |
| P1    | byte (1) | Parameter 1            | `0x00`   |
| P2    | byte (1) | Parameter 2            | `0x00`   |
| L     | byte (1) | Bytes in payload       | `0x00`   |

#### Response

| Field     | Type     | Content            | Note                     |
| --------- | -------- | ------------------ | ------------------------ |
| TARGET_ID | byte (4) | Target Id          |                          |
| OS_LEN    | byte (1) | OS version length  | `0..64`                  |
| OS        | byte (?) | OS version         | Non-terminated string    |
| FLAGS_LEN | byte (1) | Flags length       | `0`                      |
| MCU_LEN   | byte (1) | MCU version length | `0..64`                  |
| MCU       | byte (?) | MCU version        | Non-terminated string    |
| SW1-SW2   | byte (2) | Return code        | See list of return codes |

### GET_VERSION

#### Command

| Field | Type     | Content                | Expected |
| ----- | -------- | ---------------------- | -------- |
| CLA   | byte (1) | Application Identifier | `0x80`   |
| INS   | byte (1) | Instruction ID         | `0x00`   |
| P1    | byte (1) | Parameter 1            | Ignored  |
| P2    | byte (1) | Parameter 2            | Ignored  |
| L     | byte (1) | Bytes in payload       | `0`      |

#### Response

| Field   | Type     | Content          | Note                            |
| ------- | -------- | ---------------- | ------------------------------- |
| TEST    | byte (1) | Test Mode        | `0xFF` means test mode is enabled |
| MAJOR   | byte (2) | Version Major    | `0..65535`                      |
| MINOR   | byte (2) | Version Minor    | `0..65535`                      |
| PATCH   | byte (2) | Version Patch    | `0..65535`                      |
| LOCKED  | byte (1) | Device is locked |                                 |
| SW1-SW2 | byte (2) | Return code      | See list of return codes        |

### INS_GET_ADDR

#### Command

| Field         | Type      | Content                   | Expected           |
| ------------- | --------- | ------------------------- | ------------------ |
| CLA           | byte (1)  | Application Identifier    | `0x80`             |
| INS           | byte (1)  | Instruction ID            | `0x01`             |
| P1            | byte (1)  | Request User confirmation | No = `0` / Yes = `1` |
| P2            | byte (1)  | Parameter 2               | Ignored            |
| L             | byte (1)  | Bytes in payload          | `20`               |
| Path[0]       | byte (4)  | Derivation Path Data      | `0x80000000 \| 44`  |
| Path[1]       | byte (4)  | Derivation Path Data      | `0x80000000 \| 6532`|
| Path[2]       | byte (4)  | Derivation Path Data      | `0x80000000 \| 0`   |
| Account Index | byte (21) | Account Index             | ?                  |

#### Response

| Field   | Type      | Content     | Note                     |
| ------- | --------- | ----------- | ------------------------ |
| ADDR    | byte (80) | Address     |                          |
| SW1-SW2 | byte (2)  | Return code | See list of return codes |

### INS_SIGN

#### Command

| Field | Type     | Content                | Expected  |
| ----- | -------- | ---------------------- | --------- |
| CLA   | byte (1) | Application Identifier | `0x80`    |
| INS   | byte (1) | Instruction ID         | `0x02`    |
| P1    | byte (1) | Payload desc           | `0 = init`  |
|       |          |                        | `1 = add`   |
|       |          |                        | `2 = last`  |
| P2    | byte (1) | ----                   | Not used  |
| L     | byte (1) | Bytes in payload       | (Depends) |

The first packet/chunk includes only the derivation path. All other packets/chunks contain data chunks that are described below.

##### First Packet

| Field         | Type      | Content                   | Expected           |
| ------------- | --------- | ------------------------- | ------------------ |
| Path[0]       | byte (4)  | Derivation Path Data      | `0x80000000 \| 44`  |
| Path[1]       | byte (4)  | Derivation Path Data      | `0x80000000 \| 6532`|
| Path[2]       | byte (4)  | Derivation Path Data      | `0x80000000 \| 0`   |

##### Other Chunks/Packets

| Field   | Type      | Content         | Expected                  |
| ------- | --------- | --------------- | ------------------------- |
| Message | bytes (?) | Message to Sign | Hexadecimal string (utf8) |

#### Response

| Field                     | Type      | Content                               | Note                     |
| ------------------------- | --------- | ------------------------------------- | ------------------------ |
| effectHash                | byte (64) | Effect Hash                           |                          |
| spendAuthSignatureQty     | u16       | Quantity of Spend Auth Signatures     |                          |
| delegatorVoteSignatureQty | u16       | Quantity of Delegator Vote Signatures |                          |
| SW1-SW2                   | byte (2)  | Return code                           | See list of return codes |

### INS_GET_FVK

#### Command

| Field         | Type      | Content                   | Expected           |
| ------------- | --------- | ------------------------- | ------------------ |
| CLA           | byte (1)  | Application Identifier    | `0x80`             |
| INS           | byte (1)  | Instruction ID            | `0x03`             |
| P1            | byte (1)  | Request User confirmation | No = `0` / Yes = `1` |
| P2            | byte (1)  | Parameter 2               | Ignored            |
| L             | byte (1)  | Bytes in payload          | `20`               |
| Path[0]       | byte (4)  | Derivation Path Data      | `0x80000000 \| 44`  |
| Path[1]       | byte (4)  | Derivation Path Data      | `0x80000000 \| 6532`|
| Path[2]       | byte (4)  | Derivation Path Data      | `0x80000000 \| 0`   |
| Account Index | byte (20) | Account Index             | ?                  |

#### Response

| Field   | Type      | Content                      | Note                     |
| ------- | --------- | ---------------------------- | ------------------------ |
| AK      | byte (32) | Spend authorization key      |                          |
| NK      | byte (32) | Nullifier deriving key       |                          |
| SW1-SW2 | byte (2)  | Return code                  | See list of return codes |

#### Account Index

| Field          | Type       | Content          | Note         |
| -------------- | ---------- | ---------------- | ------------ |
| account        | u32        | Account          | 4 bytes      |
| randomizer     | u8 (12)    | Randomizer       | 12 bytes     |

### INS_TX_METADATA

#### Command

| Field | Type     | Content                | Expected  |
| ----- | -------- | ---------------------- | --------- |
| CLA   | byte (1) | Application Identifier | `0x80`    |
| INS   | byte (1) | Instruction ID         | `0x04`    |
| P1    | byte (1) | Payload desc           | `0 = init`  |
|       |          |                        | `1 = add`   |
|       |          |                        | `2 = last`  |
| P2    | byte (1) | ----                   | Not used  |
| L     | byte (1) | Bytes in payload       | (Depends) |

The first packet/chunk includes only the derivation path. All other packets/chunks contain data chunks that are described below.

##### First Packet

| Field         | Type      | Content                   | Expected           |
| ------------- | --------- | ------------------------- | ------------------ |
| Path[0]       | byte (4)  | Derivation Path Data      | `0x80000000 \| 44`  |
| Path[1]       | byte (4)  | Derivation Path Data      | `0x80000000 \| 6532`|
| Path[2]       | byte (4)  | Derivation Path Data      | `0x80000000 \| 0`   |

##### Other Chunks/Packets

| Field      | Type           | Content              | Expected |
| ---------- | -------------- | -------------------- | -------- |
| Length_1   | u8             | Length of Metadata_1 |          |
| Metadata_1 | bytes (Length) | Metadata_1           |          |
| ...        | ...            | ...                  |          |
| Length_n   | u8             | Length of Metadata_n |          |
| Metadata_n | bytes (Length) | Metadata_n           |          |

#### Response

| Field    | Type      | Content     | Note                     |
| -------- | --------- | ----------- | ------------------------ |
| SW1-SW2  | byte (2)  | Return code | See list of return codes |

### INS_GET_SPEND_AUTH_SIGNATURES

#### Command

| Field | Type     | Content                | Expected |
| ----- | -------- | ---------------------- | -------- |
| CLA   | byte (1) | Application Identifier | `0x80`   |
| INS   | byte (1) | Instruction ID         | `0x05`   |
| P1    | byte (1) | Index                  |          |
| P2    | byte (1) | Parameter 2            | Ignored  |

#### Response

| Field     | Type      | Content                                                    | Note                     |
| --------- | --------- | ---------------------------------------------------------- | ------------------------ |
| Signature | byte (64) | Signature of the spend action at the index specified in P1 |                          |
| SW1-SW2   | byte (2)  | Return code                                                | See list of return codes |

### INS_GET_DELEGATOR_VOTE_SIGNATURES

#### Command

| Field | Type     | Content                | Expected |
| ----- | -------- | ---------------------- | -------- |
| CLA   | byte (1) | Application Identifier | `0x80`   |
| INS   | byte (1) | Instruction ID         | `0x06`   |
| P1    | byte (1) | Index                  |          |
| P2    | byte (1) | Parameter 2            | Ignored  |

#### Response

| Field     | Type      | Content                                                             | Note                     |
| --------- | --------- | ------------------------------------------------------------------- | ------------------------ |
| Signature | byte (64) | Signature of the delegator vote action at the index specified in P1 |                          |
| SW1-SW2   | byte (2)  | Return code                                                         | See list of return codes |
