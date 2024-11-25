#include <cx.h>
#include <os.h>

#include "crypto.h"
#include "zxmacros.h"

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
