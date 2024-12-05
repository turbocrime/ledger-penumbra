/*******************************************************************************
 *   (c) 2018 - 2023 Zondax AG
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

#include <hexutils.h>

#include <iostream>
#include <vector>

#include "gmock/gmock.h"
#include "parser.h"
#include "parser_txdef.h"
#include "zxformat.h"
#include "parser_interface.h"

using namespace std;

TEST(SCALE, ReadBytes) {
    parser_context_t ctx = {0};
    parser_tx_t tx_obj = {0};
    parser_error_t err;
    zxerr_t zxerr;
    
    uint8_t buffer[6000];
    auto bufferLen =
        parseHexString(buffer, sizeof(buffer),
                       "0a47ca02440a220a20f9b7b3723506cca29bdfb54b409d7ffff96f23aaa8998101f23ff9ec4c955c861a0a08b9a1f789a28c"
                       "94de02220a08b5a8ba94d2b2ed8d032a0608d10910d109123808e0d6e5821b1223706e7964772d3138343233373332333437"
                       "3436313838363932363135333332333035341a0b0a0908c6e09fded6c3cb37");

    spend_key_bytes_t sk_bytes = {0};
    std::array<uint8_t, 32> sk_bytes_raw = {0xa1, 0xff, 0xba, 0x0c, 0x37, 0x93, 0x1f, 0x0a, 0x62, 0x61, 0x37,
                                            0x52, 0x0d, 0xa6, 0x50, 0x63, 0x2d, 0x35, 0x85, 0x3b, 0xf5, 0x91,
                                            0xb3, 0x6b, 0xb4, 0x28, 0x63, 0x0a, 0x4d, 0x87, 0xc4, 0xdc};
    std::copy(sk_bytes_raw.begin(), sk_bytes_raw.end(), sk_bytes);
    ctx.sk_bytes = &sk_bytes;

    err = parser_parse(&ctx, buffer, bufferLen, &tx_obj);
    ASSERT_EQ(err, parser_ok) << parser_getErrorDescription(err);

    // compute parameters hash
    zxerr = compute_parameters_hash(&tx_obj.parameters_plan.data_bytes, &tx_obj.plan.parameters_hash);
    ASSERT_EQ(zxerr, zxerr_ok);

    // compute action hashes
    for (uint16_t i = 0; i < tx_obj.plan.actions.qty; i++) {
        zxerr = compute_action_hash(&tx_obj.actions_plan[i], &sk_bytes, &tx_obj.plan.memo.key,
                                   &tx_obj.plan.actions.hashes[i]);
        ASSERT_EQ(zxerr, zxerr_ok);
    }

    // compute effect hash
    zxerr = compute_effect_hash(&tx_obj.plan, tx_obj.effect_hash, sizeof(tx_obj.effect_hash));
    ASSERT_EQ(zxerr, zxerr_ok);

    std::string expected =
        "09cb80cda44cd051971bcc9fa1acc71a4d4366d52b2f4ee49fdc15eb18d30dbde75373d5cfbadf467277da61907a21ec8ce3d5dd9c04af70ee6"
        "7ce4beff7bcae";
    char actual[129];
    array_to_hexstr(actual, sizeof(actual), tx_obj.effect_hash, sizeof(tx_obj.effect_hash));

    EXPECT_EQ(std::string(actual), expected);
}
