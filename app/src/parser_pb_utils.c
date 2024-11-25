/*******************************************************************************
 *  (c) 2018 - 2023 Zondax AG
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

#include "parser_pb_utils.h"

#include "parser_impl.h"
#include "parser_interface.h"
#include "zxformat.h"

bool decode_fixed_field(pb_istream_t *stream, const pb_field_t *field, void **arg) {
    if (stream->bytes_left == 0 || arg == NULL) return false;

    fixed_size_field_t *decode_arg = (fixed_size_field_t *)*arg;
    if (decode_arg == NULL || decode_arg->bytes == NULL) {
        return false;
    }

    if (decode_arg->check_size && stream->bytes_left != decode_arg->expected_size) {
        return false;
    }

    const uint8_t *first_byte = stream->state;
    uint16_t data_size = stream->bytes_left;

    decode_arg->bytes->ptr = first_byte;
    decode_arg->bytes->len = data_size;

    return true;
}

void setup_decode_fixed_field(pb_callback_t *callback, fixed_size_field_t *arg, Bytes_t *bytes, uint16_t expected_size,
                              bool check_size) {
    arg->bytes = bytes;
    arg->expected_size = expected_size;
    arg->check_size = check_size;
    callback->funcs.decode = &decode_fixed_field;
    callback->arg = arg;
}
