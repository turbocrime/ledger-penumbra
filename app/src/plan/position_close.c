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
#include "position_close.h"

#include "note.h"
#include "parser_pb_utils.h"
#include "ui_utils.h"
#include "zxformat.h"

parser_error_t decode_position_close_plan(const bytes_t *data, position_close_plan_t *position_close) {
    penumbra_core_component_dex_v1_PositionClose position_close_pb =
        penumbra_core_component_dex_v1_PositionClose_init_default;

    pb_istream_t stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t position_id_arg;
    setup_decode_fixed_field(&position_close_pb.position_id.inner, &position_id_arg, &position_close->position_id.inner,
                             ASSET_ID_LEN);

    if (!pb_decode(&stream, penumbra_core_component_dex_v1_PositionClose_fields, &position_close_pb)) {
        return parser_position_close_plan_error;
    }

    position_close->has_position_id = position_close_pb.has_position_id;

    return parser_ok;
}

parser_error_t position_close_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    *num_items = 1;
    return parser_ok;
}

parser_error_t position_close_getItem(const parser_context_t *ctx, const position_close_plan_t *position_close,
                                      uint8_t actionIdx, char *outKey, uint16_t outKeyLen, char *outVal, uint16_t outValLen,
                                      uint8_t pageIdx, uint8_t *pageCount) {
    parser_error_t err = parser_no_data;
    if (position_close == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }

    char bufferUI[POSITION_CLOSE_DISPLAY_MAX_LEN] = {0};

    snprintf(outKey, outKeyLen, "Action_%d", actionIdx);
    CHECK_ERROR(position_close_printValue(ctx, position_close, bufferUI, sizeof(bufferUI)));
    pageString(outVal, outValLen, bufferUI, pageIdx, pageCount);

    return parser_ok;
}

parser_error_t position_close_printValue(const parser_context_t *ctx, const position_close_plan_t *position_close,
                                         char *outVal, uint16_t outValLen) {
    if (ctx == NULL || position_close == NULL || outVal == NULL) {
        return parser_no_data;
    }

    if (outValLen < POSITION_CLOSE_DISPLAY_MAX_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    // add action title
    snprintf(outVal, outValLen, "PositionClose Position ID ");
    uint16_t written_value = strlen(outVal);

    MEMCPY(outVal + written_value, position_close->position_id.inner.ptr, position_close->position_id.inner.len);

    CHECK_ERROR(encodePositionId(position_close->position_id.inner.ptr, position_close->position_id.inner.len,
                                 outVal + written_value, outValLen - written_value));

    return parser_ok;
}
