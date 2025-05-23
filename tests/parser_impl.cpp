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
        "0abe020abb020aa8010a300a0a08cbe5fdba84e7fff00712220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf32"
        "2ca96"
        "a101220931e59aa772ba57d44dc85d301dfb75dc1636b96fe57fa25c249a0b944d18ab71a520a50890bc98e3698aa4578e419b028da567"
        "2e627"
        "c280d8b06166f4c42d5366bccf1fcf3b296cd61e8d744a21f75f2fb697183e18595d8a79008539d8fb138b405db09db65cc42d54c0e772"
        "e5d42"
        "d5f20b52f10a19f8ecd8ff5161a202d6bdd168c08c681ed138083d9feece3e281dd293e1b8286ce7ba07007a579042220c6b3f83c86a08"
        "7f881"
        "249186bbc65c32ca269ffa9cd7f91d2b6f1fcd011d0b012a20d30887aaf837dedb1943bb44e5fa4006ec01b9475acc1ff4f3053608583a"
        "19053"
        "2204dc0e0a6902ac14c6585da44589c701e3a6bf74f1676e8043aa1d040d35cbb03124b123b65757170777863766f71696a746f7464656"
        "a666a"
        "787968766e767a656d62696c2d33313936373534393637323436373231343831353837363438301a0c0a0a08a7dcb5f99dc5888109");

    err = parser_parse(&ctx, buffer, bufferLen, &tx_obj);
    ASSERT_EQ(err, parser_ok) << parser_getErrorDescription(err);

    err = parser_computeEffectHash(&ctx);
    ASSERT_EQ(err, parser_ok) << parser_getErrorDescription(err);

    std::string expected =
        "bd6419d9a090c7461f9d5749b126fbb291288071f3fe39abfb24eca6e3e6c3daafea1a7ca63269393c5e50bf4256b847853e70d6e0ae6b"
        "ef5fa"
        "891f3e2c1310f";
    char actual[129];
    array_to_hexstr(actual, sizeof(actual), tx_obj.effect_hash, sizeof(tx_obj.effect_hash));

    EXPECT_EQ(std::string(actual), expected);
}
