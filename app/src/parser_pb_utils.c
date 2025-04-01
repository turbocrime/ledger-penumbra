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

bool decode_fixed_field(pb_istream_t *stream, __Z_UNUSED const pb_field_t *field, void **arg) {
    if (stream->bytes_left == 0 || arg == NULL) return false;

    fixed_size_field_t *decode_arg = (fixed_size_field_t *)*arg;
    if (decode_arg == NULL || decode_arg->bytes == NULL) {
        return false;
    }

    if (stream->bytes_left != decode_arg->expected_size) {
        return false;
    }

    decode_arg->bytes->ptr = stream->state;
    decode_arg->bytes->len = stream->bytes_left;

    return true;
}

bool decode_variable_field(pb_istream_t *stream, __Z_UNUSED const pb_field_t *field, void **arg) {
    if (stream->bytes_left == 0 || arg == NULL) return false;

    variable_size_field_t *decode_arg = (variable_size_field_t *)*arg;
    if (decode_arg == NULL || decode_arg->bytes == NULL) {
        return false;
    }

    decode_arg->bytes->ptr = stream->state;
    decode_arg->bytes->len = stream->bytes_left;

    return true;
}

bool decode_variable_field_array(pb_istream_t *stream, __Z_UNUSED const pb_field_t *field, void **arg) {
    if (stream->bytes_left == 0 || arg == NULL) return false;

    variable_size_field_array_t *decode_arg = (variable_size_field_array_t *)*arg;
    if (decode_arg == NULL || decode_arg->bytes_array == NULL) {
        return false;
    }

    if (decode_arg->filled_count >= decode_arg->array_size) {
        return false;  // Array is full
    }

    // Fill the next available slot in the array
    decode_arg->bytes_array[decode_arg->filled_count].ptr = stream->state;
    decode_arg->bytes_array[decode_arg->filled_count].len = stream->bytes_left;
    decode_arg->filled_count++;

    return true;
}

void setup_decode_fixed_field(pb_callback_t *callback, fixed_size_field_t *arg, bytes_t *bytes, uint16_t expected_size) {
    arg->bytes = bytes;
    arg->expected_size = expected_size;
    callback->funcs.decode = &decode_fixed_field;
    callback->arg = arg;
}

void setup_decode_variable_field(pb_callback_t *callback, variable_size_field_t *arg, bytes_t *bytes) {
    arg->bytes = bytes;
    callback->funcs.decode = &decode_variable_field;
    callback->arg = arg;
}

void setup_decode_variable_field_array(pb_callback_t *callback, variable_size_field_array_t *arg, bytes_t *bytes_array,
                                       size_t array_size) {
    arg->bytes_array = bytes_array;
    arg->array_size = array_size;
    arg->filled_count = 0;
    callback->funcs.decode = &decode_variable_field_array;
    callback->arg = arg;
}

parser_error_t extract_data_from_tag(const bytes_t *in, bytes_t *out, uint32_t tag) {
    const uint8_t *start = NULL;
    const uint8_t *end = NULL;
    bool eof = false;

    pb_istream_t scan_stream = pb_istream_from_buffer(in->ptr, in->len);
    pb_wire_type_t wire_type;
    uint32_t tag_internal;
    while (pb_decode_tag(&scan_stream, &wire_type, &tag_internal, &eof) && !eof) {
        if (tag_internal == tag) {
            start = scan_stream.state;
            if (!pb_skip_field(&scan_stream, wire_type)) {
                return parser_unexpected_error;
            }
            end = scan_stream.state;
            break;
        } else {
            if (!pb_skip_field(&scan_stream, wire_type)) {
                return parser_unexpected_error;
            }
        }
    }

    if (!start || !end) {
        return parser_unexpected_error;
    }

    out->ptr = start + 1;
    out->len = end - start - 1;

    return parser_ok;
}
