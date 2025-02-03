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

#include "delegator_vote.h"

#include "note.h"
#include "parser_pb_utils.h"
#include "rslib.h"
#include "ui_utils.h"
#include "zxformat.h"

static void vote_to_string(const uint8_t vote, char *outVal, uint16_t outValLen) {
    switch (vote) {
        case VOTE_UNSPECIFIED:
            snprintf(outVal, outValLen, "Unspecified");
            break;
        case VOTE_YES:
            snprintf(outVal, outValLen, "Yes");
            break;
        case VOTE_NO:
            snprintf(outVal, outValLen, "No");
            break;
        case VOTE_ABSTAIN:
            snprintf(outVal, outValLen, "Abstain");
            break;
    }
}

parser_error_t decode_delegator_vote_plan(const bytes_t *data, delegator_vote_plan_t *delegator_vote) {
    penumbra_core_component_governance_v1_DelegatorVotePlan delegator_vote_plan =
        penumbra_core_component_governance_v1_DelegatorVotePlan_init_default;

    pb_istream_t stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t randomizer_arg;
    setup_decode_fixed_field(&delegator_vote_plan.randomizer, &randomizer_arg, &delegator_vote->randomizer, 32);

    // staked_note
    fixed_size_field_t address_inner_arg, asset_id_arg, rseed_arg;
    setup_decode_fixed_field(&delegator_vote_plan.staked_note.address.inner, &address_inner_arg,
                             &delegator_vote->staked_note.address.inner, 80);
    setup_decode_fixed_field(&delegator_vote_plan.staked_note.value.asset_id.inner, &asset_id_arg,
                             &delegator_vote->staked_note.value.asset_id.inner, ASSET_ID_LEN);
    setup_decode_fixed_field(&delegator_vote_plan.staked_note.rseed, &rseed_arg, &delegator_vote->staked_note.rseed,
                             RSEED_LEN);

    if (!pb_decode(&stream, penumbra_core_component_governance_v1_DelegatorVotePlan_fields, &delegator_vote_plan)) {
        return parser_delegator_vote_plan_error;
    }

    delegator_vote->proposal = delegator_vote_plan.proposal;
    delegator_vote->start_position = delegator_vote_plan.start_position;
    delegator_vote->has_vote = delegator_vote_plan.has_vote;
    if (delegator_vote_plan.has_vote) {
        delegator_vote->vote = delegator_vote_plan.vote.vote;
        if (delegator_vote_plan.vote.vote == VOTE_UNSPECIFIED) {
            return parser_unexpected_error;
        }
    }
    delegator_vote->has_staked_note = delegator_vote_plan.has_staked_note;
    if (delegator_vote_plan.has_staked_note) {
        delegator_vote->staked_note.has_value = delegator_vote_plan.staked_note.has_value;
        if (delegator_vote_plan.staked_note.has_value) {
            delegator_vote->staked_note.value.has_amount = delegator_vote_plan.staked_note.value.has_amount;
            if (delegator_vote_plan.staked_note.value.has_amount) {
                delegator_vote->staked_note.value.amount.lo = delegator_vote_plan.staked_note.value.amount.lo;
                delegator_vote->staked_note.value.amount.hi = delegator_vote_plan.staked_note.value.amount.hi;
            }
            delegator_vote->staked_note.value.has_asset_id = delegator_vote_plan.staked_note.value.has_asset_id;
        }
        delegator_vote->staked_note.has_address = delegator_vote_plan.staked_note.has_address;
    }

    delegator_vote->staked_note_position = delegator_vote_plan.staked_note_position;

    delegator_vote->has_unbonded_amount = delegator_vote_plan.has_unbonded_amount;
    if (delegator_vote_plan.has_unbonded_amount) {
        delegator_vote->unbonded_amount.lo = delegator_vote_plan.unbonded_amount.lo;
        delegator_vote->unbonded_amount.hi = delegator_vote_plan.unbonded_amount.hi;
    }

    return parser_ok;
}

parser_error_t delegator_vote_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    *num_items = 1;
    return parser_ok;
}

parser_error_t delegator_vote_getItem(const parser_context_t *ctx, const delegator_vote_plan_t *delegator_vote,
                                      uint8_t actionIdx, char *outKey, uint16_t outKeyLen, char *outVal, uint16_t outValLen,
                                      uint8_t pageIdx, uint8_t *pageCount) {
    parser_error_t err = parser_no_data;
    if (delegator_vote == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }

    char bufferUI[DELEGATE_DISPLAY_MAX_LEN] = {0};

    snprintf(outKey, outKeyLen, "Action_%d", actionIdx);
    CHECK_ERROR(delegator_vote_printValue(ctx, delegator_vote, bufferUI, sizeof(bufferUI)));
    pageString(outVal, outValLen, bufferUI, pageIdx, pageCount);

    return parser_ok;
}

parser_error_t delegator_vote_printValue(const parser_context_t *ctx, const delegator_vote_plan_t *delegator_vote,
                                         char *outVal, uint16_t outValLen) {
    if (ctx == NULL || delegator_vote == NULL || outVal == NULL) {
        return parser_no_data;
    }

    if (outValLen < DELEGATE_DISPLAY_MAX_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    // add action title
    snprintf(outVal, outValLen, "DelegatorVote on Proposal ");
    uint16_t written_value = strlen(outVal);

    // add proposal
    uint64_to_str(outVal + written_value, outValLen - written_value, delegator_vote->proposal);
    written_value = strlen(outVal);

    // add vote
    snprintf(outVal + written_value, outValLen - written_value, " Vote ");
    written_value = strlen(outVal);
    vote_to_string(delegator_vote->vote, outVal + written_value, outValLen - written_value);
    written_value = strlen(outVal);

    // add voting power
    snprintf(outVal + written_value, outValLen - written_value, " Voting Power: ");
    written_value = strlen(outVal);

    // add unbonded amount
    static const uint8_t default_asset_id[ASSET_ID_LEN] = STAKING_TOKEN_ASSET_ID_BYTES;
    value_t local_value = {.amount.hi = delegator_vote->unbonded_amount.hi,
                           .amount.lo = delegator_vote->unbonded_amount.lo,
                           .asset_id.inner.ptr = default_asset_id,
                           .asset_id.inner.len = ASSET_ID_LEN,
                           .has_amount = true,
                           .has_asset_id = true};
    CHECK_ERROR(printValue(ctx, &local_value, &ctx->tx_obj->parameters_plan.chain_id, true, outVal + written_value,
                           outValLen - written_value));

    return parser_ok;
}