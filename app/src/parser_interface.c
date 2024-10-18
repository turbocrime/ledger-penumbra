/*******************************************************************************
 *  (c) 2018 - 2024 Zondax AG
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
#include "parser_interface.h"

#include <string.h>

#include "keys_def.h"
#include "rslib.h"
#include "zxformat.h"

void print_buffer_interface(Bytes_t *buffer, const char *title) {
#if defined(LEDGER_SPECIFIC)
    ZEMU_LOGF(50, "%s\n", title);
    char print[700] = {0};
    array_to_hexstr(print, sizeof(print), buffer->ptr, buffer->len);
    ZEMU_LOGF(700, "%s\n", print);
#else
    printf("%s: ", title);
    for (uint16_t i = 0; i < buffer->len; i++) {
        printf("%02x", buffer->ptr[i]);
    }
    printf("\n");
#endif
}

parser_error_t compute_transaction_plan(transaction_plan_t *plan) {
    if (plan == NULL) return parser_unexpected_error;

    uint8_t output[300] = {0};
    if (rs_compute_transaction_plan(plan, output, sizeof(output)) != parser_ok) {
        return parser_unexpected_error;
    }

    // TODO: only for testing
    Bytes_t output_bytes;
    output_bytes.ptr = output;
    output_bytes.len = 300;
    print_buffer_interface(&output_bytes, "output_bytes");

    return parser_ok;
}
