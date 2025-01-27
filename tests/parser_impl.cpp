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
        "0a27b203240a220a20d236a0af60136307f49dcf9d21d4d5bfc8025b4ada87a819cb0d6de82681b8e0121e08c6a70d120a70656e756d6272612"
        "d311a0c0a0a0889aaa69f92f5d9c3042afa030ad5030a520a50caef612cb3e0aa9799cfad490583bac0ca8240ec27085df3409c6fb8721a7490"
        "c7c4b1e566574b4278f257c6aa49cda0cd4d2d0ab42a889e0b40a183a7915312ebdde08f869eddac9d1445c387c3e35f12fe02205a716c31644"
        "b7057343262535220204f20783051596d4a6c394448644e76325171203266302030307a423753306a515920316b207135435046733020736820"
        "68544e793935696e2057716d30476558667a342020334f30346d536e45334e44366737652020306f3753697245386a4936204620336c3542776"
        "c55204c6267506a6632203420204b415838363252342051367120206d3039642071204420354f205a776948356820706d7a205336316a204146"
        "74566333322078203278205975303532206132584f53206c6b3553304120372020533820775a78334d786f36676d75543538205576767854376"
        "22020472038344e4320635430474655636a6f2063384d346d2074663938323651656f6e496b73505220336c5565436957736976374f56366875"
        "5639416342346d37207320207a59202056313679203439384c207420594c4f307473463662594e694636343174557948507632206920674c6f6"
        "1505a6652324a346e202055634556323820203143204620584d68386320411220618b73a44f2dd0b8b5e444e7fc7df7bef6cdf33b1a4bad802c"
        "4a48d5021a2678");

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
        "9dd2c5349b49dc367bc7e51d639773dd1850f68f885ad33d2cb0417a790c854eb865387f99738cdaf951a03ddb2319d81563448c99f9061baa5"
        "0d6e8423b7f97";
    char actual[129];
    array_to_hexstr(actual, sizeof(actual), tx_obj.effect_hash, sizeof(tx_obj.effect_hash));

    EXPECT_EQ(std::string(actual), expected);
}
