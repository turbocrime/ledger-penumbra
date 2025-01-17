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
        "0aaf01f201ab010aa8010a640a18120a08e0ec81d6c1b2b59b021a0a08fefa92ed958bd4a50d12480a220a2029ea9c2f3371f6a487e7e95c247"
        "041f4a356f983eb064e5d2b3bcf322ca96a1012220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a10122000"
        "000000000000000000000000000000000000000000000000000000000000001a02080122180a0a08c7f2cdf9f2e5fb8407120a08a0a7b4ef80c"
        "ca0e20c280112311221626a2d3834333039393235303336313933333433373530393738363435343930351a0c0a0a08a2d5eb8ec9aab7cb092a"
        "87040ae2030a520a505ba4d5f0ecdfe87e4d76f6c6adc54c0cebd5a29d5e3428409326920df48f88f9be38e96472777a885fdfe3a18ce9afbfb"
        "27c0b262feda69c2146b334c0c918ad463a8468d15bb09664848ebe5e5fe30c128b03205863636e55203675752030627033617a207932392020"
        "58556572677420647147202033497431692039553920205374766b593950687642207347784e2079546c7962353474753751674e72522038472"
        "06173473237346f554e2073354b34436235203320564a6c3220744271456e79772078652047204b686f205146206a3920384e42764733793269"
        "6c474a6a323339302044207a3269476e6c20453220695345563935656734636e572038662064306c315a46324c2032203072203137632039664"
        "64f207a6e446b393520555438594e4c3878204866792052726437703876364220386e374170312020333349202035206e79322049566c386f20"
        "69205639302020356f4620656453783920203834206f2066376e6b365677446920207a202038344b31392052445934206955516f44394245517"
        "73746474f2075614f6a6632374a467320206431756a62207439506d2035783546303035206c6968596454382020476e46463973487647434f4a"
        "52596f33384d6a6a475951207573472031352075344a70787365671220f79e4ad6b1d9f0bc7f268a0af4ce8cb14c34421bef47ae65a3f5428bf"
        "54b035e");

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
        "54814cada1609da600716b8259c9e3206eec9b337e9217590b948ab7e0908180577e43063d40abd2fcce0c90aa4a33ed32fd6050bdf27f5bc4f"
        "9576147bf4c82";
    char actual[129];
    array_to_hexstr(actual, sizeof(actual), tx_obj.effect_hash, sizeof(tx_obj.effect_hash));

    EXPECT_EQ(std::string(actual), expected);
}
