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
#include "protobuf/penumbra/core/transaction/v1/transaction.pb.h"
#include "rslib.h"
#include "zxformat.h"

zxerr_t compute_effect_hash(transaction_plan_t *plan, uint8_t *effect_hash, uint16_t effect_hash_len) {
    if (plan == NULL || effect_hash == NULL) return zxerr_unknown;

    if (rs_compute_effect_hash(plan, effect_hash, effect_hash_len) != parser_ok) {
        return zxerr_unknown;
    }

    return zxerr_ok;
}

zxerr_t compute_parameters_hash(bytes_t *parameters_bytes, hash_t *output) {
    if (parameters_bytes == NULL || output == NULL) return zxerr_unknown;

    if (rs_parameter_hash(parameters_bytes, (uint8_t *)output, 64) != parser_ok) {
        return zxerr_unknown;
    }

    return zxerr_ok;
}

zxerr_t compute_action_hash(action_t *action, bytes_t *memo_key, hash_t *output) {
    if (action == NULL || output == NULL) return zxerr_unknown;

    switch (action->action_type) {
        case penumbra_core_transaction_v1_ActionPlan_spend_tag:
            if (rs_spend_action_hash(&action->action.spend, (uint8_t *)output, 64) != parser_ok) {
                return zxerr_encoding_failed;
            }
            break;
        case penumbra_core_transaction_v1_ActionPlan_output_tag:
            if (rs_output_action_hash(&action->action.output, memo_key, (uint8_t *)output, 64) != parser_ok) {
                return zxerr_encoding_failed;
            }
            break;
        case penumbra_core_transaction_v1_ActionPlan_swap_tag:
            if (rs_swap_action_hash(&action->action.swap, (uint8_t *)output, 64) != parser_ok) {
                return zxerr_encoding_failed;
            }
            break;
        case penumbra_core_transaction_v1_ActionPlan_delegate_tag:
        case penumbra_core_transaction_v1_ActionPlan_undelegate_tag:
        case penumbra_core_transaction_v1_ActionPlan_ics20_withdrawal_tag:
            if (rs_generic_action_hash(&action->action_data, action->action_type, (uint8_t *)output, 64) != parser_ok) {
                return zxerr_encoding_failed;
            }
            break;
        default:
            return zxerr_unknown;
    }

    return zxerr_ok;
}
