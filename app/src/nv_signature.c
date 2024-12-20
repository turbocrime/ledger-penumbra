/*******************************************************************************
 *   (c) 2018 -2022 Zondax AG
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 ********************************************************************************/

#include "nv_signature.h"
#include "zxmacros.h"

#define MAX_SIGNATURES 16

#define SIGNATURE_BUFFER_LEN MAX_SIGNATURES *sizeof(signature_t)

// Flash
typedef struct {
    uint8_t buffer[SIGNATURE_BUFFER_LEN];
} storage_t;

typedef struct {
    uint8_t *data;
    size_t size;
    size_t pos;
} flash_state_t;

#if defined(TARGET_NANOS) || defined(TARGET_NANOX) || defined(TARGET_NANOS2) || defined(TARGET_STAX) || defined(TARGET_FLEX)
// SpendAuth signature buffer
storage_t NV_CONST N_spend_data_impl __attribute__((aligned(64)));
#define N_spend_data (*(NV_VOLATILE storage_t *)PIC(&N_spend_data_impl))
#endif

flash_state_t spend_state;

void nv_signature_init() {
    spend_state.data = (uint8_t *)N_spend_data.buffer;
    spend_state.size = SIGNATURE_BUFFER_LEN;
    spend_state.pos = 0;
}

void nv_signature_reset() {
    spend_state.pos = 0;
    spend_state.size = 0;
}

size_t nv_write_signature(const signature_t signature, signature_type_t type) {
    // For now only Spend signature types are supported
    if (type != Spend) {
        return 0;
    }

    size_t len = sizeof(signature_t);
    if (spend_state.size - spend_state.pos >= len) {
        MEMCPY_NV(spend_state.data + spend_state.pos, (void *)signature, len);
        spend_state.pos += len;
        return len;
    } else {
        return 0;
    }
}

size_t nv_get_signature(uint16_t index, signature_t *signature, signature_type_t type) {
    // For now only Spend signature types are supported
    if (type != Spend) {
        return 0;
    }

    size_t len = sizeof(signature_t);
    size_t offset = len * index;

    if (spend_state.size < offset + len) {
        return 0;
    }

    MEMCPY(signature, spend_state.data + offset, len);

    return len;
}

size_t nv_num_signatures(signature_type_t type) {
    // For now only Spend signature types are supported
    if (type != Spend) {
        return 0;
    }
    return spend_state.pos / sizeof(signature_t);
}
