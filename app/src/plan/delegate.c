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

#include "note.h"
#include "parser_pb_utils.h"
#include "rslib.h"
#include "ui_utils.h"
#include "zxformat.h"

parser_error_t decode_delegate_plan(const bytes_t *data, delegate_plan_t *delegate) {
    penumbra_core_component_stake_v1_Delegate delegate_plan = penumbra_core_component_stake_v1_Delegate_init_default;

    pb_istream_t spend_stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t validator_identity_arg;
    setup_decode_fixed_field(&delegate_plan.validator_identity.ik, &validator_identity_arg, &delegate->validator_identity.ik,
                             32);

    if (!pb_decode(&spend_stream, penumbra_core_component_stake_v1_Delegate_fields, &delegate_plan)) {
        return parser_delegate_plan_error;
    }

    delegate->has_validator_identity = delegate_plan.has_validator_identity;
    delegate->epoch_index = delegate_plan.epoch_index;
    if (delegate_plan.has_unbonded_amount) {
        delegate->unbonded_amount.lo = delegate_plan.unbonded_amount.lo;
        delegate->unbonded_amount.hi = delegate_plan.unbonded_amount.hi;
    }
    if (delegate_plan.has_delegation_amount) {
        delegate->delegation_amount.lo = delegate_plan.delegation_amount.lo;
        delegate->delegation_amount.hi = delegate_plan.delegation_amount.hi;
    }

    return parser_ok;
}
