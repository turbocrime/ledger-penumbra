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
        "0adf028202db020a180a0a08c48687cae1a9bf9203120a08ace285e5a28debf40312220a204b3ded9a00383b8dc5e8ceb20c3d3755417c5168b"
        "372403c307847526a223b191a480a220a20be7e67051f8968163a6bcdeba31c1bcb9198e6a4b8504c0766c9181eeafaf30a12220a209f03c391"
        "0ab73af2e70701930fe9e6bf521f6f61849850a0347ad4fbef41b11120bdd4cfb6bdf5c2bcb2012a300a0a08ebbbfbb4dac6cef30212220a20b"
        "e7e67051f8968163a6bcdeba31c1bcb9198e6a4b8504c0766c9181eeafaf30a2a300a0a08d3e1d5d89d8cbb9e0112220a201d6d84ab75195520"
        "6db68530522fcc52d13baebd9453bfd41f9d346f2a7b38072a300a0a08baea9f89e8b9cef80c12220a209f03c3910ab73af2e70701930fe9e6b"
        "f521f6f61849850a0347ad4fbef41b1112a300a0a08aad1a9d499f5c7e30b12220a201d6d84ab751955206db68530522fcc52d13baebd9453bf"
        "d41f9d346f2a7b3807122008f6d434120c7a68646f2d363038383130381a0c0a0a08efa6cab6cca7efe601");

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
        "4b86c0923927796d0931bb03a25407a26ebca3b5a0200357f3ab8de28d958adde326cab74d73a278f5ac97700f58d4534c0b956e20c27e277ba"
        "ae43e3fd3dd5c";
    char actual[129];
    array_to_hexstr(actual, sizeof(actual), tx_obj.effect_hash, sizeof(tx_obj.effect_hash));

    EXPECT_EQ(std::string(actual), expected);
}
