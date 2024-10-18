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

#include "parser_impl.h"

#include "parser_interface.h"
#include "pb_common.h"
#include "pb_decode.h"
#include "protobuf/penumbra/core/transaction/v1/transaction.pb.h"
#include "zxformat.h"

static bool decode_field(pb_istream_t *stream, const pb_field_t *field, void **arg);
static bool decode_action(pb_istream_t *stream, const pb_field_t *field, void **arg);
static bool decode_detection_data(pb_istream_t *stream, const pb_field_t *field, void **arg);
static void setup_decode_field(pb_callback_t *callback, decode_memo_field_arg_t *arg, Bytes_t *bytes, uint16_t expected_size,
                               bool check_size);
static parser_error_t extract_data_from_tag(Bytes_t *in, Bytes_t *out, uint32_t tag);

static uint16_t actions_qty = 0;
static uint16_t detection_data_qty = 0;

void print_buffer(Bytes_t *buffer, const char *title) {
#if defined(LEDGER_SPECIFIC)
    ZEMU_LOGF(50, "%s\n", title);
    char print[1000] = {0};
    array_to_hexstr(print, sizeof(print), buffer->ptr, buffer->len);
    ZEMU_LOGF(1000, "%s\n", print);
#else
    printf("%s %d: ", title, buffer->len);
    for (uint16_t i = 0; i < buffer->len; i++) {
        printf("%02x", buffer->ptr[i]);
    }
    printf("\n");
#endif
}

void print_string(const char *str) {
#if defined(LEDGER_SPECIFIC)
    ZEMU_LOGF(100, "%s\n", str);
#else
    printf("%s\n", str);
#endif
}

bool decode_field(pb_istream_t *stream, const pb_field_t *field, void **arg) {
    if (stream->bytes_left == 0 || arg == NULL) return false;

    decode_memo_field_arg_t *decode_arg = (decode_memo_field_arg_t *)*arg;
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

void setup_decode_field(pb_callback_t *callback, decode_memo_field_arg_t *arg, Bytes_t *bytes, uint16_t expected_size,
                        bool check_size) {
    arg->bytes = bytes;
    arg->expected_size = expected_size;
    arg->check_size = check_size;
    callback->funcs.decode = &decode_field;
    callback->arg = arg;
}

bool decode_action(pb_istream_t *stream, const pb_field_t *field, void **arg) {
    penumbra_core_transaction_v1_ActionPlan action = penumbra_core_transaction_v1_ActionPlan_init_default;

    action_t *decode_arg = (action_t *)*arg;
    if (decode_arg == NULL) {
        return false;
    }

    if (actions_qty >= ACTIONS_QTY) {
        return false;
    }

    const uint8_t *first_byte = stream->state;
    uint16_t data_size = stream->bytes_left;
    decode_arg[actions_qty].action.ptr = first_byte + 2;
    decode_arg[actions_qty].action.len = data_size - 2;

    if (!pb_decode(stream, penumbra_core_transaction_v1_ActionPlan_fields, &action)) {
        return false;
    }
    decode_arg[actions_qty].action_type = action.which_action;
    switch (action.which_action) {
        case penumbra_core_transaction_v1_ActionPlan_spend_tag:
            print_string("Spend action detected \n");
            break;
        case penumbra_core_transaction_v1_ActionPlan_output_tag:
            print_string("Output action detected\n");
            break;
        case penumbra_core_transaction_v1_ActionPlan_swap_tag:
            print_string("Swap action detected\n");
            break;
        case penumbra_core_transaction_v1_ActionPlan_swap_claim_tag:
            print_string("SwapClaim action detected\n");
            break;
        case penumbra_core_transaction_v1_ActionPlan_validator_definition_tag:
            print_string("Delegate action detected\n");
            break;
        case penumbra_core_transaction_v1_ActionPlan_ibc_relay_action_tag:
            print_string("Undelegate action detected\n");
            break;
        case penumbra_core_transaction_v1_ActionPlan_proposal_submit_tag:
            print_string("UndelegateClaim action detected\n");
            break;
        default:
            print_string("Unknown action detected\n");
            return false;
    }
    actions_qty++;

    return true;
}

bool decode_detection_data(pb_istream_t *stream, const pb_field_t *field, void **arg) {
    if (stream->bytes_left == 0 || arg == NULL) return false;

    if (detection_data_qty >= DETECTION_DATA_QTY) {
        return false;
    }

    penumbra_core_transaction_v1_CluePlan cluePlan = penumbra_core_transaction_v1_CluePlan_init_default;
    decode_memo_field_arg_t rseed_arg, address_inner_arg, address_alt_bech32m_arg;
    clue_plan_t *clue_plan_arg = (clue_plan_t *)*arg;

    setup_decode_field(&cluePlan.rseed, &rseed_arg, &clue_plan_arg[detection_data_qty].rseed, RSEED_SIZE, true);
    setup_decode_field(&cluePlan.address.inner, &address_inner_arg, &clue_plan_arg[detection_data_qty].address.inner,
                       MEMO_ADDRESS_INNER_SIZE, true);
    setup_decode_field(&cluePlan.address.alt_bech32m, &address_alt_bech32m_arg,
                       &clue_plan_arg[detection_data_qty].address.alt_bech32m, 0, false);

    if (!pb_decode(stream, penumbra_core_transaction_v1_CluePlan_fields, &cluePlan)) {
        return false;
    }

    clue_plan_arg[detection_data_qty].precision_bits = cluePlan.precision_bits;

    detection_data_qty++;
    return true;
}

parser_error_t extract_data_from_tag(Bytes_t *in, Bytes_t *out, uint32_t tag) {
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

// parser_error_t extract_data_array_from_tag(Bytes_t *in, Bytes_t out[], uint32_t tag) {
//     const uint8_t *start = NULL;
//     const uint8_t *end = NULL;
//     bool eof = false;

//     pb_istream_t scan_stream = pb_istream_from_buffer(in->ptr, in->len);
//     pb_wire_type_t wire_type;
//     uint32_t tag_internal;
//     size_t out_index = 0;

//     while (pb_decode_tag(&scan_stream, &wire_type, &tag_internal, &eof) && !eof) {
//         if (tag_internal == tag) {
//             start = scan_stream.state;
//             if (!pb_skip_field(&scan_stream, wire_type)) {
//                 return parser_unexpected_error;
//             }
//             end = scan_stream.state;

//             if (!start || !end) {
//                 return parser_unexpected_error;
//             }

//             out[out_index].ptr = start + 1;
//             out[out_index].len = end - start - 1;
//             out_index++;
//         } else {
//             if (!pb_skip_field(&scan_stream, wire_type)) {
//                 return parser_unexpected_error;
//             }
//         }
//     }

//     return parser_ok;
// }

// parser_error_t compute_detection_data(Bytes_t *detection_data, parser_tx_t *v) {
//     uint16_t detection_data_bytes = DETECTION_DATA_SIZE;
//     uint16_t clue_plans = detection_data->len / detection_data_bytes;

//     if (clue_plans > 1) {
//         if ((detection_data->ptr[0] + 1 == clue_plans) || (clue_plans * detection_data_bytes + 1 == detection_data->len))
//         {
//             for (uint16_t i = 0; i < clue_plans; i++) {
//                 v->plan.detection_data.clue_plans[i].ptr = detection_data->ptr + (i * detection_data_bytes + 1);
//                 v->plan.detection_data.clue_plans[i].len = DETECTION_DATA_SIZE;
//             }
//         } else {
//             return parser_unexpected_error;
//         }
//     } else {
//         v->plan.detection_data.clue_plans[0].ptr = detection_data->ptr;
//         v->plan.detection_data.clue_plans[0].len = detection_data->len;
//     }

//     return parser_ok;
// }

// parser_error_t compute_actions(Bytes_t actions[], parser_tx_t *v) {
//     for (uint16_t i = 0; i < ACTIONS_QTY; i++) {
//         if (actions[i].len > 2) {
//             v->plan.actions[i].action_type = actions[i].ptr[1] >> 3;
//             // TODO: check if we have to parser each action
//             v->plan.actions[i].action.ptr = actions[i].ptr + 2;
//             v->plan.actions[i].action.len = actions[i].len - 2;
//         }
//     }

//     return parser_ok;
// }

parser_error_t _read(parser_context_t *c, parser_tx_t *v) {
    Bytes_t data;
    data.ptr = c->buffer;
    data.len = c->bufferLen;
    actions_qty = 0;
    detection_data_qty = 0;

    penumbra_core_transaction_v1_TransactionPlan request = penumbra_core_transaction_v1_TransactionPlan_init_default;
    decode_memo_field_arg_t memo_key_arg, memo_text_arg, memo_return_address_inner_arg, memo_return_address_alt_bech32m_arg;

    // memo callbacks
    setup_decode_field(&request.memo.key, &memo_key_arg, &v->plan.memo.key, MEMO_KEY_SIZE, true);
    setup_decode_field(&request.memo.plaintext.text, &memo_text_arg, &v->plan.memo.plaintext.text, 0, false);
    setup_decode_field(&request.memo.plaintext.return_address.inner, &memo_return_address_inner_arg,
                       &v->plan.memo.plaintext.return_address.inner, MEMO_ADDRESS_INNER_SIZE, true);
    setup_decode_field(&request.memo.plaintext.return_address.alt_bech32m, &memo_return_address_alt_bech32m_arg,
                       &v->plan.memo.plaintext.return_address.alt_bech32m, 0, false);

    // actions callbacks
    request.actions.funcs.decode = &decode_action;
    request.actions.arg = &v->plan.actions;

    // detection data callbacks
    request.detection_data.clue_plans.funcs.decode = &decode_detection_data;
    request.detection_data.clue_plans.arg = &v->plan.detection_data.clue_plans;

    pb_istream_t stream = pb_istream_from_buffer(c->buffer, c->bufferLen);
    CHECK_APP_CANARY()
    const bool status = pb_decode(&stream, penumbra_core_transaction_v1_TransactionPlan_fields, &request);
    if (!status) {
        // TODO: improve handling errors from callbacks
        if (actions_qty == ACTIONS_QTY) {
            return parser_actions_overflow;
        }
        if (detection_data_qty == DETECTION_DATA_QTY) {
            return parser_detection_data_overflow;
        }
        return parser_unexpected_error;
    }

    // get transaction parameters
    extract_data_from_tag(&data, &v->plan.transaction_parameters.parameters,
                          penumbra_core_transaction_v1_TransactionPlan_transaction_parameters_tag);
    print_buffer(&v->plan.transaction_parameters.parameters, "real transaction parameters");

    // print detection data
    for (uint16_t i = 0; i < DETECTION_DATA_QTY; i++) {
        print_buffer(&v->plan.detection_data.clue_plans[i].address.inner, "real detection data address inner");
        print_buffer(&v->plan.detection_data.clue_plans[i].address.alt_bech32m, "real detection data address alt bech32m");
        print_buffer(&v->plan.detection_data.clue_plans[i].rseed, "real detection data rseed");
        // printf("precision bits: %lu\n", v->plan.detection_data.clue_plans[i].precision_bits);
    }

    // print actions
    for (uint16_t i = 0; i < ACTIONS_QTY; i++) {
        print_buffer(&v->plan.actions[i].action, "real actions");
    }

    // print memo
    print_buffer(&v->plan.memo.key, "real memo key");
    print_buffer(&v->plan.memo.plaintext.text, "real memo plaintext text");
    print_buffer(&v->plan.memo.plaintext.return_address.inner, "real memo return address inner");
    print_buffer(&v->plan.memo.plaintext.return_address.alt_bech32m, "real memo return address alt bech32m");

    compute_transaction_plan(&v->plan);

    return parser_unexpected_error;
}

const char *parser_getErrorDescription(parser_error_t err) {
    switch (err) {
        case parser_ok:
            return "No error";
        case parser_no_data:
            return "No more data";
        case parser_init_context_empty:
            return "Initialized empty context";
        case parser_unexpected_buffer_end:
            return "Unexpected buffer end";
        case parser_unexpected_version:
            return "Unexpected version";
        case parser_unexpected_characters:
            return "Unexpected characters";
        case parser_unexpected_field:
            return "Unexpected field";
        case parser_duplicated_field:
            return "Unexpected duplicated field";
        case parser_value_out_of_range:
            return "Value out of range";
        case parser_unexpected_chain:
            return "Unexpected chain";
        case parser_missing_field:
            return "missing field";

        case parser_display_idx_out_of_range:
            return "display index out of range";
        case parser_display_page_out_of_range:
            return "display page out of range";
        case parser_actions_overflow:
            return "actions overflow";
        case parser_detection_data_overflow:
            return "detection data overflow";

        default:
            return "Unrecognized error code";
    }
}
