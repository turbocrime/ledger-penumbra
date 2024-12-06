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
#include "zxformat.h"
#include "note.h"
#include "ui_utils.h"
#include "rslib.h"

parser_error_t decode_ics20_withdrawal_plan(const bytes_t *data, ics20_withdrawal_plan_t *withdrawal) {
    penumbra_core_component_ibc_v1_Ics20Withdrawal withdrawal_plan = penumbra_core_component_ibc_v1_Ics20Withdrawal_init_default;

    pb_istream_t withdrawal_stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t return_address_arg;
    setup_decode_fixed_field(&withdrawal_plan.return_address.inner, &return_address_arg, &withdrawal->return_address.inner, 80);

    // Set up variable size fields
    variable_size_field_t denom_arg, destination_chain_address_arg, source_channel_arg;
    setup_decode_variable_field(&withdrawal_plan.denom.denom, &denom_arg, &withdrawal->denom.inner);
    setup_decode_variable_field(&withdrawal_plan.destination_chain_address, &destination_chain_address_arg, &withdrawal->destination_chain_address);
    setup_decode_variable_field(&withdrawal_plan.source_channel, &source_channel_arg, &withdrawal->source_channel);

    if (!pb_decode(&withdrawal_stream, penumbra_core_component_ibc_v1_Ics20Withdrawal_fields, &withdrawal_plan)) {
        return parser_ics20_withdrawal_plan_error;
    }

    withdrawal->has_amount = withdrawal_plan.has_amount;
    if (withdrawal_plan.has_amount) {
        withdrawal->amount.lo = withdrawal_plan.amount.lo;
        withdrawal->amount.hi = withdrawal_plan.amount.hi;
    }
    withdrawal->has_denom = withdrawal_plan.has_denom;
    withdrawal->has_return_address = withdrawal_plan.has_return_address;
    withdrawal->has_timeout_height = withdrawal_plan.has_timeout_height;
    if (withdrawal_plan.has_timeout_height) {
        withdrawal->timeout_height.revision_number = withdrawal_plan.timeout_height.revision_number;
        withdrawal->timeout_height.revision_height = withdrawal_plan.timeout_height.revision_height;
    }
    withdrawal->timeout_time = withdrawal_plan.timeout_time;
    withdrawal->use_compat_address = withdrawal_plan.use_compat_address;

    return parser_ok;
}
