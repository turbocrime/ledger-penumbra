#include "note.h"

#include "coin.h"
#include "constants.h"
#include "known_assets.h"
#include "tx_metadata.h"
#include "ui_utils.h"
#include "zxformat.h"

bool is_zero_amount(const value_t *value) { return value->amount.hi == 0 && value->amount.lo == 0; }

parser_error_t printValue(const parser_context_t *ctx, const value_t *value, const bytes_t *chain_id,
                          const bool format_amount, char *outVal, uint16_t outValLen) {
    if (ctx == NULL || value == NULL || outVal == NULL || chain_id == NULL) {
        return parser_no_data;
    }

    if (outValLen < VALUE_DISPLAY_MAX_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    char amount_str[U128_STR_MAX_LEN] = {0};

    // convert to string note.amount
    CHECK_ERROR(uint128_to_str(amount_str, U128_STR_MAX_LEN, value->amount.hi, value->amount.lo))

    // lookup at asset table
    const asset_info_t *known_asset = NULL;
    if (value->has_asset_id) {
        known_asset = asset_info_from_table(value->asset_id.inner.ptr);
    }

    // There are three cases:
    // Case 1: Known assets (decimal + space + symbol)
    // Case 2: Base denom (integer + space + denom) taken from transaction
    // Case 3: Bech32 fallback (integer + space + bech32 of asset_id)
    // where asset_id is unknown and not metadata was provided for it

    // Case 1: Known assets
    if (known_asset != NULL) {
        // check if chain id is penumbra-1
        if (strncmp((const char *)chain_id->ptr, DEFAULT_CHAIN_ID, chain_id->len) == 0) {
            if (format_amount) {
                return printNumber(amount_str, value->has_amount, COIN_AMOUNT_DECIMAL_PLACES, known_asset->symbol, "",
                                   outVal, outValLen);
            } else {
                return printNumber(amount_str, value->has_amount, 0, known_asset->symbol, "", outVal, outValLen);
            }
        } else {
            // check in denom the format data
            bool was_printed = false;
            CHECK_ERROR(tryPrintDenom(ctx, value, amount_str, outVal, outValLen, &was_printed));
            if (was_printed) {
                return parser_ok;
            }

            // Case 3: Bech32 fallback (integer + space + bech32 of asset_id)
            CHECK_ERROR(printFallback(value, amount_str, value->has_amount, outVal, outValLen));
            return parser_ok;
        }
    }

    // Case 2: Base denom (integer + space + denom) taken from transaction
    // for this we use the parser_context_t to access the transaction metadata
    // if not found, we default to case 3
    bool was_printed = false;
    CHECK_ERROR(tryPrintDenom(ctx, value, amount_str, outVal, outValLen, &was_printed));
    if (was_printed) {
        return parser_ok;
    }

    // Case 3: Bech32 fallback (integer + space + bech32 of asset_id)
    CHECK_ERROR(printFallback(value, amount_str, value->has_amount, outVal, outValLen));

    return parser_ok;
}

parser_error_t printFee(const parser_context_t *ctx, const value_t *value, const bytes_t *chain_id, char *outVal,
                        uint16_t outValLen) {
    if (ctx == NULL || value == NULL || outVal == NULL || chain_id == NULL) {
        return parser_no_data;
    }

    if (outValLen < VALUE_DISPLAY_MAX_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    // when asset id is not present, use the default asset id
    value_t local_value = *value;
    if (!value->has_asset_id) {
        static const uint8_t default_asset_id[ASSET_ID_LEN] = STAKING_TOKEN_ASSET_ID_BYTES;
        local_value.asset_id.inner.ptr = default_asset_id;
        local_value.asset_id.inner.len = ASSET_ID_LEN;
        local_value.has_asset_id = true;
    }

    CHECK_ERROR(printValue(ctx, &local_value, chain_id, true, outVal, outValLen));

    return parser_ok;
}

parser_error_t tryPrintDenom(const parser_context_t *ctx, const value_t *value, const char *amount_str, char *outVal,
                             uint16_t outValLen, bool *was_printed) {
    if (ctx == NULL || value == NULL || outVal == NULL || amount_str == NULL) {
        return parser_no_data;
    }

    char denom[MAX_DENOM_LEN + 1] = {0};
    *was_printed = false;

    uint8_t trace_len = 0;
    if (value->asset_id.inner.ptr != NULL && value->asset_id.inner.len != 0) {
        trace_len = metadata_getDenom(&ctx->tx_metadata[0], ctx->tx_metadata_len, &value->asset_id.inner, denom,
                                      MAX_DENOM_LEN + 1);
    }

    if (trace_len != 0) {
        // We found denom trace in provided transaction metadata
        snprintf(outVal, outValLen - 1, "%s", amount_str);
        uint16_t written = strlen(outVal);
        if (written >= outValLen - 1) {
            return parser_unexpected_buffer_end;
        }

        // Space
        outVal[written] = ' ';
        written += 1;
        // check we have space for denomination path
        if (written + trace_len >= outValLen - 1) {
            return parser_unexpected_buffer_end;
        }

        MEMCPY(&outVal[written], denom, trace_len);
        written += trace_len;
        outVal[written] = '\0';

        *was_printed = true;
        return parser_ok;
    }

    return parser_ok;
}

parser_error_t printFallback(const value_t *value, const char *amount_str, bool has_amount, char *outVal,
                             uint16_t outValLen) {
    uint16_t written = 0;
    if (has_amount) {
        snprintf(outVal, outValLen - 1, "%s", amount_str);
        written = strlen(outVal);
        if (written >= outValLen - 1) {
            return parser_unexpected_buffer_end;
        }
        // Space
        outVal[written] = ' ';
        written += 1;
    }

    return printAssetId(value->asset_id.inner.ptr, value->asset_id.inner.len, outVal + written,
                        outValLen - written - 1);
}

parser_error_t printNumber(const char *amount, bool has_amount, uint8_t decimalPlaces, const char *postfix,
                           const char *prefix, char *outVal, uint16_t outValLen) {
    char amount_trimmed[VALUE_DISPLAY_MAX_LEN] = {0};
    if (has_amount) {
        if (strcmp(amount, "0") == 0) {
            snprintf(amount_trimmed, VALUE_DISPLAY_MAX_LEN, "%s", "0");
        } else {
            if (fpstr_to_str(amount_trimmed, VALUE_DISPLAY_MAX_LEN, amount, decimalPlaces) != 0) {
                return parser_unexpected_value;
            }
        }

        number_inplace_trimming(amount_trimmed, 1);

        // add space
        size_t fpstr_len = strlen(amount_trimmed);
        amount_trimmed[fpstr_len] = ' ';
    }

    if (z_str3join(amount_trimmed, VALUE_DISPLAY_MAX_LEN, prefix, postfix) != zxerr_ok) {
        return parser_unexpected_buffer_end;
    }

    snprintf(outVal, outValLen, "%s", amount_trimmed);

    return parser_ok;
}

parser_error_t printAssetIdFromValue(const parser_context_t *ctx, const value_t *value, const bytes_t *chain_id,
                                     char *outVal, uint16_t outValLen) {
    if (ctx == NULL || value == NULL || outVal == NULL || chain_id == NULL) {
        return parser_no_data;
    }

    if (outValLen < ASSET_ID_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    // lookup at asset table
    const asset_info_t *known_asset = NULL;
    if (value->has_asset_id) {
        known_asset = asset_info_from_table(value->asset_id.inner.ptr);
    }

    // Case 1: Known assets
    if (known_asset != NULL) {
        // check if chain id is penumbra-1
        if (strncmp((const char *)chain_id->ptr, DEFAULT_CHAIN_ID, chain_id->len) == 0) {
            snprintf(outVal, outValLen, "%s", known_asset->symbol);
        } else {
            CHECK_ERROR(printAssetId(value->asset_id.inner.ptr, value->asset_id.inner.len, outVal, outValLen));
        }

        return parser_ok;
    }

    // Case 2: Base denom
    char denom[MAX_DENOM_LEN + 1] = {0};

    uint8_t trace_len = 0;
    if (value->asset_id.inner.ptr != NULL && value->asset_id.inner.len != 0) {
        trace_len = metadata_getDenom(&ctx->tx_metadata[0], ctx->tx_metadata_len, &value->asset_id.inner, denom,
                                      MAX_DENOM_LEN + 1);
    }

    if (trace_len != 0) {
        MEMCPY(&outVal, denom, trace_len);
        outVal[trace_len] = '\0';

        return parser_ok;
    }

    // Case 3: Bech32 fallback
    CHECK_ERROR(printAssetId(value->asset_id.inner.ptr, value->asset_id.inner.len, outVal, outValLen));

    return parser_ok;
}