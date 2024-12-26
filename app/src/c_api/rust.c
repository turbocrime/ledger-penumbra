#include <zxmacros.h>

#include "parser_common.h"
#include "zxerror.h"

#if defined(LEDGER_SPECIFIC)
#include <cx.h>
#include <os.h>

#include "crypto.h"
#include "crypto_helper.h"

zxerr_t crypto_extractSpendingKeyBytes(uint8_t *key_bytes, uint32_t key_bytes_len);

// Function to compute BLAKE2b hash with personalization
zxerr_t blake2b_hash_with_personalization(const uint8_t *input, size_t input_len, uint8_t *output, size_t output_len,
                                          const uint8_t *label, size_t label_len) {
    cx_blake2b_t hash_context;

    // unsigned char *salt = NULL;
    // unsigned int salt_len = 0;

    zxerr_t error = zxerr_invalid_crypto_settings;

    // no salt
    CATCH_CXERROR(cx_blake2b_init2_no_throw(&hash_context, output_len * 8, NULL, 0, (uint8_t *)label, label_len));

    CATCH_CXERROR(cx_hash_no_throw(&hash_context.header, CX_LAST, input, input_len, output, output_len));

catch_cx_error:
    return error;
}

zxerr_t crypto_getFvkBytes(uint8_t *fvk, uint16_t len) {
    zxerr_t error = zxerr_invalid_crypto_settings;

    if (len < FVK_LEN) {
        return error;
    }

    uint16_t cmdResponseLen = 0;
    return crypto_fillKeys(fvk, len, &cmdResponseLen);
}
#else

// This to support cpp tests
zxerr_t crypto_getFvkBytes(uint8_t *sk, uint16_t len) {
    // bytes of fvk for zemu seed
    uint8_t fvk_bytes_raw[FVK_LEN] = {0x92, 0xc3, 0xe7, 0x68, 0xd3, 0xec, 0xf0, 0xf2, 0xc4, 0xd9, 0x3d, 0x87, 0x9d,
                                      0xbc, 0x16, 0x22, 0x6f, 0xe8, 0x54, 0x04, 0x43, 0xa8, 0x21, 0x6d, 0x6b, 0x09,
                                      0x3d, 0x86, 0x84, 0x86, 0x5a, 0x06, 0x3a, 0x35, 0xee, 0x29, 0xcc, 0xcf, 0x93,
                                      0x14, 0x9d, 0xfa, 0x56, 0x5e, 0xa6, 0x93, 0xaa, 0x5c, 0xd3, 0x6d, 0xc5, 0xcf,
                                      0x8a, 0xdf, 0xf1, 0x50, 0x81, 0x03, 0x8d, 0x31, 0xe7, 0x96, 0x58, 0x0b};
    MEMCPY(sk, fvk_bytes_raw, FVK_LEN);

    return zxerr_ok;
}

#endif