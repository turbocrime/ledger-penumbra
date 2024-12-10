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
        "0abe020abb020aa8010a300a0a089bd0d0a0c4a0b0d70d12220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96"
        "a1012209f5822139d82485e474f2e9acb0fc7549d16fc64f39176048cdf74a5ac38f91c1a520a50890bc98e3698aa4578e419b028da5672e627"
        "c280d8b06166f4c42d5366bccf1fcf3b296cd61e8d744a21f75f2fb697183e18595d8a79008539d8fb138b405db09db65cc42d54c0e772e5d42"
        "d5f20b52f10e2c0e0c99797121a20c4db44519ed152a93d3c0adead64ff37dc948a491a10fee23abb7ce6833eda0222204180bd3e351b2044cb"
        "6a70d1879ae141d41189ab7fbb4fe7def02027578c7a012a2059f86ceb345416bf07f27fa6df7c57b366e72f3fb8b6490cf1921b1eb8719c113"
        "220ceba4f740fa139b424baee9f720812c28abde87421c1459e07541464a0465509121a120a70656e756d6272612d311a0c0a0a08abcbd699ff"
        "a0c1900c2acb020aa6020a520a508d5b14d34c66c974180c1c3537b4c6167759244fc34a3fcd582f6e937a48aa27939fdb08733c64a49a59461"
        "977b6a45e5201fd087fef594b117f3e6628e1889ecc382d5d5dfc8a383fa51ff84119bc8512cf017a20383842204f736d334a6f3020204b7135"
        "67204820354b5a4a35203736536251774a6e71316450306b33664152303620654257205a345720315837734c4820577420363420336c6d4e536"
        "b30495073664b3020204c20203951204b357336204c466a206571202041204c396d20204f4e2020577849202043656d333944584a2073393050"
        "6350203139694368316757783132376642204f51334a32205a3820396f3020534e6e4c20505a667a69204a46393348482020734520484376664"
        "94b62532067355075675a43206877654d1220d6b269dbe8d6e04bdbba2025285d956864c723c3932ba608db6fd433a194731b");

    spend_key_bytes_t sk_bytes = {0};
    std::array<uint8_t, 32> sk_bytes_raw = {0xa1, 0xff, 0xba, 0x0c, 0x37, 0x93, 0x1f, 0x0a, 0x62, 0x61, 0x37,
                                            0x52, 0x0d, 0xa6, 0x50, 0x63, 0x2d, 0x35, 0x85, 0x3b, 0xf5, 0x91,
                                            0xb3, 0x6b, 0xb4, 0x28, 0x63, 0x0a, 0x4d, 0x87, 0xc4, 0xdc};
    std::copy(sk_bytes_raw.begin(), sk_bytes_raw.end(), sk_bytes);

    err = parser_parse(&ctx, buffer, bufferLen, &tx_obj);
    ASSERT_EQ(err, parser_ok) << parser_getErrorDescription(err);

    // compute parameters hash
    zxerr = compute_parameters_hash(&tx_obj.parameters_plan.data_bytes, &tx_obj.plan.parameters_hash);
    ASSERT_EQ(zxerr, zxerr_ok);

    // compute action hashes
    for (uint16_t i = 0; i < tx_obj.plan.actions.qty; i++) {
        zxerr =
            compute_action_hash(&tx_obj.actions_plan[i], &sk_bytes, &tx_obj.plan.memo.key, &tx_obj.plan.actions.hashes[i]);
        ASSERT_EQ(zxerr, zxerr_ok);
    }

    // compute effect hash
    zxerr = compute_effect_hash(&tx_obj.plan, tx_obj.effect_hash, sizeof(tx_obj.effect_hash));
    ASSERT_EQ(zxerr, zxerr_ok);

    std::string expected =
        "00292f5a3cee6d88cde30afe07a0e4e1b3d4c2f240844567e816c838b59fd94e7b494f64fbc5b7a97f087c017a162a78378cf953f782c5666d8"
        "b209aae228d74";
    char actual[129];
    array_to_hexstr(actual, sizeof(actual), tx_obj.effect_hash, sizeof(tx_obj.effect_hash));

    EXPECT_EQ(std::string(actual), expected);
}
