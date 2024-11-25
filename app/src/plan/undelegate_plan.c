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
#include "parser_pb_utils.h"
#include "pb_common.h"
#include "pb_decode.h"
#include "protobuf/penumbra/core/transaction/v1/transaction.pb.h"
#include "zxformat.h"

parser_error_t decode_undelegate_plan(const bytes_t *data, undelegate_plan_t *undelegate) {
    penumbra_core_component_stake_v1_Undelegate undelegate_plan = penumbra_core_component_stake_v1_Undelegate_init_default;

    pb_istream_t spend_stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t validator_identity_arg;
    setup_decode_fixed_field(&undelegate_plan.validator_identity.ik, &validator_identity_arg,
                             &undelegate->validator_identity.ik, 32);

    if (!pb_decode(&spend_stream, penumbra_core_component_stake_v1_Undelegate_fields, &undelegate_plan)) {
        return parser_undelegate_plan_error;
    }

    undelegate->has_validator_identity = undelegate_plan.has_validator_identity;
    undelegate->start_epoch_index = undelegate_plan.start_epoch_index;
    if (undelegate_plan.has_unbonded_amount) {
        undelegate->unbonded_amount.lo = undelegate_plan.unbonded_amount.lo;
        undelegate->unbonded_amount.hi = undelegate_plan.unbonded_amount.hi;
    }
    if (undelegate_plan.has_delegation_amount) {
        undelegate->delegation_amount.lo = undelegate_plan.delegation_amount.lo;
        undelegate->delegation_amount.hi = undelegate_plan.delegation_amount.hi;
    }
    if (undelegate_plan.has_from_epoch) {
        undelegate->from_epoch.index = undelegate_plan.from_epoch.index;
        undelegate->from_epoch.start_height = undelegate_plan.from_epoch.start_height;
    }

    return parser_ok;
}
