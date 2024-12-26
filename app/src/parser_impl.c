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

#include "delegate.h"
#include "ics20_withdrawal.h"
#include "output.h"
#include "parameters.h"
#include "parser_interface.h"
#include "parser_pb_utils.h"
#include "pb_common.h"
#include "pb_decode.h"
#include "protobuf/penumbra/core/transaction/v1/transaction.pb.h"
#include "spend.h"
#include "swap.h"
#include "undelegate.h"
#include "zxformat.h"

static bool decode_action(pb_istream_t *stream, const pb_field_t *field, void **arg);
static bool decode_detection_data(pb_istream_t *stream, const pb_field_t *field, void **arg);

static uint16_t actions_qty = 0;
static uint16_t detection_data_qty = 0;
static parser_error_t decode_error = parser_ok;

#define CHECK_ACTION_ERROR(__CALL)       \
    {                                    \
        decode_error = __CALL;           \
        CHECK_APP_CANARY()               \
        if (decode_error != parser_ok) { \
            return false;                \
        }                                \
    }

bool decode_action(pb_istream_t *stream, const pb_field_t *field, void **arg) {
    if (arg == NULL || *arg == NULL) {
        return false;
    }

    action_t *decode_arg = (action_t *)*arg;
    if (decode_arg == NULL) {
        return false;
    }

    if (actions_qty >= ACTIONS_QTY) {
        decode_error = parser_actions_overflow;
        return false;
    }

    penumbra_core_transaction_v1_ActionPlan action = penumbra_core_transaction_v1_ActionPlan_init_default;

    bytes_t action_data = {.ptr = stream->state + 3, .len = stream->bytes_left - 3};
    bytes_t ics20_withdrawal_data = {.ptr = stream->state + 4, .len = stream->bytes_left - 4};

    if (!pb_decode(stream, penumbra_core_transaction_v1_ActionPlan_fields, &action)) {
        return false;
    }
    decode_arg[actions_qty].action_type = action.which_action;
    switch (action.which_action) {
        case penumbra_core_transaction_v1_ActionPlan_spend_tag:
            decode_arg[actions_qty].action_data = action_data;
            CHECK_ACTION_ERROR(decode_spend_plan(&action_data, &decode_arg[actions_qty].action.spend));
            break;
        case penumbra_core_transaction_v1_ActionPlan_output_tag:
            decode_arg[actions_qty].action_data = action_data;
            CHECK_ACTION_ERROR(decode_output_plan(&action_data, &decode_arg[actions_qty].action.output));
            break;
        case penumbra_core_transaction_v1_ActionPlan_delegate_tag:
            decode_arg[actions_qty].action_data = action_data;
            CHECK_ACTION_ERROR(decode_delegate_plan(&action_data, &decode_arg[actions_qty].action.delegate));
            break;
        case penumbra_core_transaction_v1_ActionPlan_undelegate_tag:
            decode_arg[actions_qty].action_data = action_data;
            CHECK_ACTION_ERROR(decode_undelegate_plan(&action_data, &decode_arg[actions_qty].action.undelegate));
            break;
        case penumbra_core_transaction_v1_ActionPlan_ics20_withdrawal_tag:
            decode_arg[actions_qty].action_data = ics20_withdrawal_data;
            CHECK_ACTION_ERROR(
                decode_ics20_withdrawal_plan(&ics20_withdrawal_data, &decode_arg[actions_qty].action.ics20_withdrawal));
            break;
        case penumbra_core_transaction_v1_ActionPlan_swap_tag:
            decode_arg[actions_qty].action_data = action_data;
            CHECK_ACTION_ERROR(decode_swap_plan(&action_data, &decode_arg[actions_qty].action.swap));
            break;
        default:
            return false;
    }
    actions_qty++;

    return true;
}

bool decode_detection_data(pb_istream_t *stream, const pb_field_t *field, void **arg) {
    if (stream->bytes_left == 0 || arg == NULL) return false;

    if (detection_data_qty >= DETECTION_DATA_QTY) {
        decode_error = parser_detection_data_overflow;
        return false;
    }

    penumbra_core_transaction_v1_CluePlan cluePlan = penumbra_core_transaction_v1_CluePlan_init_default;
    fixed_size_field_t rseed_arg, address_inner_arg;
    variable_size_field_t address_alt_bech32m_arg;
    clue_plan_t *clue_plan_arg = (clue_plan_t *)*arg;

    setup_decode_fixed_field(&cluePlan.rseed, &rseed_arg, &clue_plan_arg[detection_data_qty].rseed, RSEED_SIZE);
    setup_decode_fixed_field(&cluePlan.address.inner, &address_inner_arg, &clue_plan_arg[detection_data_qty].address.inner,
                             MEMO_ADDRESS_INNER_SIZE);
    setup_decode_variable_field(&cluePlan.address.alt_bech32m, &address_alt_bech32m_arg,
                                &clue_plan_arg[detection_data_qty].address.alt_bech32m);

    if (!pb_decode(stream, penumbra_core_transaction_v1_CluePlan_fields, &cluePlan)) {
        return false;
    }

    clue_plan_arg[detection_data_qty].precision_bits = cluePlan.precision_bits;

    detection_data_qty++;
    return true;
}

parser_error_t _read(parser_context_t *c, parser_tx_t *v) {
    bytes_t data = {0};
    data.ptr = c->buffer;
    data.len = c->bufferLen;
    actions_qty = 0;
    detection_data_qty = 0;

    penumbra_core_transaction_v1_TransactionPlan request = penumbra_core_transaction_v1_TransactionPlan_init_default;
    fixed_size_field_t memo_key_arg, memo_return_address_inner_arg;
    variable_size_field_t memo_text_arg, memo_return_address_alt_bech32m_arg;

    // memo callbacks
    setup_decode_fixed_field(&request.memo.key, &memo_key_arg, &v->plan.memo.key, MEMO_KEY_SIZE);
    setup_decode_variable_field(&request.memo.plaintext.text, &memo_text_arg, &v->plan.memo.plaintext.text);
    setup_decode_fixed_field(&request.memo.plaintext.return_address.inner, &memo_return_address_inner_arg,
                             &v->plan.memo.plaintext.return_address.inner, MEMO_ADDRESS_INNER_SIZE);
    setup_decode_variable_field(&request.memo.plaintext.return_address.alt_bech32m, &memo_return_address_alt_bech32m_arg,
                                &v->plan.memo.plaintext.return_address.alt_bech32m);

    // actions callbacks
    request.actions.funcs.decode = &decode_action;
    request.actions.arg = &v->actions_plan;

    // parameters callbacks
    fixed_size_field_t parameter_asset_id_arg;
    variable_size_field_t parameter_chain_id_arg;
    setup_decode_variable_field(&request.transaction_parameters.chain_id, &parameter_chain_id_arg,
                                &v->parameters_plan.chain_id);
    setup_decode_fixed_field(&request.transaction_parameters.fee.asset_id.inner, &parameter_asset_id_arg,
                             &v->parameters_plan.fee.asset_id.inner, ASSET_ID_LEN);

    // detection data callbacks
    request.detection_data.clue_plans.funcs.decode = &decode_detection_data;
    request.detection_data.clue_plans.arg = &v->plan.detection_data.clue_plans;

    // reset error
    decode_error = parser_ok;

    pb_istream_t stream = pb_istream_from_buffer(c->buffer, c->bufferLen);
    CHECK_APP_CANARY()
    const bool status = pb_decode(&stream, penumbra_core_transaction_v1_TransactionPlan_fields, &request);
    if (!status) {
        if (decode_error != parser_ok) {
            return decode_error;
        }
        return parser_unexpected_error;
    }

    v->plan.has_parameters = request.has_transaction_parameters;
    if (request.has_transaction_parameters) {
        CHECK_ERROR(decode_parameters(&data, &request.transaction_parameters, &v->parameters_plan));
    }
    v->plan.has_memo = request.has_memo;
    v->plan.has_detection_data = request.has_detection_data;
    v->plan.actions.qty = actions_qty;

    return parser_ok;
}

const char *parser_getErrorDescription(parser_error_t err) {
    switch (err) {
        case parser_ok:
            return "No error";
        case parser_no_data:
            return "No more data";
        case parser_init_context_empty:
            return "Initialized empty context";
        case parser_display_idx_out_of_range:
            return "Display index out of range";
        case parser_display_page_out_of_range:
            return "Display page out of range";
        case parser_unexpected_error:
            return "Unexpected error";
        case parser_invalid_hash_mode:
            return "Invalid hash mode";
        case parser_invalid_signature:
            return "Invalid signature";
        case parser_invalid_pubkey_encoding:
            return "Invalid public key encoding";
        case parser_invalid_address_version:
            return "Invalid address version";
        case parser_invalid_address_length:
            return "Invalid address length";
        case parser_invalid_type_id:
            return "Invalid type ID";
        case parser_invalid_codec:
            return "Invalid codec";
        case parser_invalid_threshold:
            return "Invalid threshold";
        case parser_invalid_network_id:
            return "Invalid network ID";
        case parser_invalid_chain_id:
            return "Invalid chain ID";
        case parser_invalid_ascii_value:
            return "Invalid ASCII value";
        case parser_invalid_timestamp:
            return "Invalid timestamp";
        case parser_invalid_staking_amount:
            return "Invalid staking amount";
        case parser_operation_overflows:
            return "Operation overflows";
        case parser_invalid_path:
            return "Invalid path";
        case parser_invalid_length:
            return "Invalid length";
        case parser_too_many_outputs:
            return "Too many outputs";
        case parser_unexpected_data:
            return "Unexpected data";
        case parser_invalid_clue_key:
            return "Invalid clue key";
        case parser_invalid_tx_key:
            return "Invalid transaction key";
        case parser_invalid_fq:
            return "Invalid Fq";
        case parser_invalid_detection_key:
            return "Invalid detection key";
        case parser_invalid_fvk:
            return "Invalid FVK";
        case parser_invalid_ivk:
            return "Invalid IVK";
        case parser_invalid_key_len:
            return "Invalid key length";
        case parser_invalid_action_type:
            return "Invalid action type";
        case parser_invalid_precision:
            return "Invalid precision";
        case parser_precision_too_large:
            return "Precision too large";
        case parser_clue_creation_failed:
            return "Clue creation failed";
        case parser_invalid_asset_id:
            return "Invalid asset ID";
        case parser_unexpected_type:
            return "Unexpected type";
        case parser_unexpected_method:
            return "Unexpected method";
        case parser_unexpected_buffer_end:
            return "Unexpected buffer end";
        case parser_unexpected_value:
            return "Unexpected value";
        case parser_unexpected_number_items:
            return "Unexpected number of items";
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
        case parser_invalid_address:
            return "Invalid address";
        case parser_unexpected_chain:
            return "Unexpected chain";
        case parser_missing_field:
            return "Missing field";
        case paser_unknown_transaction:
            return "Unknown transaction";
        case parser_detection_data_overflow:
            return "Detection data overflow";
        case parser_actions_overflow:
            return "Actions overflow";
        case parser_spend_plan_error:
            return "Spend plan error";
        case parser_output_plan_error:
            return "Output plan error";
        case parser_delegate_plan_error:
            return "Delegate plan error";
        case parser_undelegate_plan_error:
            return "Undelegate plan error";
        case parser_ics20_withdrawal_plan_error:
            return "ICS20 withdrawal plan error";
        case parser_swap_plan_error:
            return "Swap plan error";
        case parser_invalid_metadata:
            return "Invalid metadata";
        default:
            return "Unrecognized error code";
    }
}
