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
        "0aae02aa01aa0208bc96b47f10141a02080122a8010a300a0a08c2aac09d8fc0d09d0c12220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a1012207c14e7434fde0abeccbc2579e58eeb65045e538b14cad708c988075b9fc0df661a520a507616f6c402371db1fa79eca16f1892132bbc1ea65e133fa67388049719f62f45c36fe666cc95ecc4444f6561a36d30fa6aad47a89032c8966f05a7cb098f9fd9ee392d0d337f3c35a33284ed4317f392281e320a08b5b081c9afa9b6c4033a208ee3fae74bc73f0107e4f6fbb6a58be4326a0d6991af104f825b8ee4387a6b0142203ad8f590111f2259243cc440cd5aebcce1f96c719095b6d68f6111ceb1f1ae054a20f59b1272a4d5ba8bea095233e51b392fe32ec7b97667aa44bf9f89321810ec10121a120a70656e756d6272612d311a0c0a0a08f9a195a2e1bee7a70b2a9f020afa010a520a5049620e19635ebc01f681246b4afffd1b09c951f932913e58422365d9716308721b9155943ca17e212004b1c4c887618effef4a3a356c819fbd3a51a29fe306aecf0d182f1a1e63a95710764220b00ef012a3015152206220497862756b20466b7420774c374e20677176205820585074725720672043206b3843506138536871554273204136696574353774372032625543393459382020554c20472062363264365246366f34583152512078436173653638556c203935684e587637353750375a77426f375663205265654341456139344156206d3459617531322020346220697572706d754d4e2074453375397349365642394f12208580fdd3426c1e8c2a98c9d28e00dce9d8801773b77a38547e2e944968ec5bda");

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
        "be610c7e26f4bddaf1ea52d70e2b7f75d66fc3110fddcd424a24b48433b4ee5a7d9706dbcceb8bdb9999715f14c01d3bc1e16b6a741fcd737752dedf3ffe805a";
    char actual[129];
    array_to_hexstr(actual, sizeof(actual), tx_obj.effect_hash, sizeof(tx_obj.effect_hash));

    EXPECT_EQ(std::string(actual), expected);
}
