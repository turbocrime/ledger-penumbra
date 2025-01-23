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
#include "parser_interface.h"
#include "parser_txdef.h"
#include "zxformat.h"

using namespace std;

TEST(SCALE, ReadBytes) {
    parser_context_t ctx = {0};
    parser_tx_t tx_obj = {0};
    parser_error_t err;
    zxerr_t zxerr;

    uint8_t buffer[6000];
    auto bufferLen = parseHexString(
        buffer, sizeof(buffer),
        "0aa701aa03a3010aa0010a300a0a088cc0b0b4f4beb4a30712220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca"
        "96a1012220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a101a0a08c581b4d0d5dbc7ec05220a0886e68daa"
        "c0b4a6f6092897da857c3098da857c38b79dcba30142200c5f7b2df77aabf7f38d26618485f51c51337c159528d013408c7601cdb310a60aa90"
        "1aa03a5010aa2010a300a0a08ebea9d83c9fb9bc40512220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a10"
        "12220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a101a0a08faa6cda885b0bea702220a08d99f9ca9e0c0c"
        "3dc0428a4a5bca40130a5a5bca40138d5e7c3b50342209138cb498ea2007358ecedf67cd752780c1b9de4e0be7c57da199107ded3226f121a12"
        "0a70656e756d6272612d311a0c0a0a08cdada9db83bc938804");

    err = parser_parse(&ctx, buffer, bufferLen, &tx_obj);
    ASSERT_EQ(err, parser_ok) << parser_getErrorDescription(err);

    // compute parameters hash
    zxerr = compute_parameters_hash(&tx_obj.parameters_plan.data_bytes, &tx_obj.plan.parameters_hash);
    ASSERT_EQ(zxerr, zxerr_ok);

    // compute action hashes
    for (uint16_t i = 0; i < tx_obj.plan.actions.qty; i++) {
        zxerr = compute_action_hash(&tx_obj.actions_plan[i], &tx_obj.plan.memo.key, &tx_obj.plan.actions.hashes[i]);
        ASSERT_EQ(zxerr, zxerr_ok);
    }

    // compute effect hash
    zxerr = compute_effect_hash(&tx_obj.plan, tx_obj.effect_hash, sizeof(tx_obj.effect_hash));
    ASSERT_EQ(zxerr, zxerr_ok);

    std::string expected =
        "8a5be23d8930ab2cf2a5457846e0030f99dfe6e35c79c83068bf09dc057ce427827ae3c3cb95ad5e1115d21025af4e5e53f8e152c0ea2bad2a0"
        "f5604894a84f5";
    char actual[129];
    array_to_hexstr(actual, sizeof(actual), tx_obj.effect_hash, sizeof(tx_obj.effect_hash));

    EXPECT_EQ(std::string(actual), expected);
}
