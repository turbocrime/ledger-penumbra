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
        "0abe020abb020aa8010a300a0a08caedc786f98cc3e50312220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96"
        "a10122042b70d0c3e32c65670ca200dbd3c3a9a9dd59ef896fee3579025b841050359431a520a50890bc98e3698aa4578e419b028da5672e627"
        "c280d8b06166f4c42d5366bccf1fcf3b296cd61e8d744a21f75f2fb697183e18595d8a79008539d8fb138b405db09db65cc42d54c0e772e5d42"
        "d5f20b52f109d94b78f8798391a20e5a37679976d378f1dabe0fd608211081bb9c43b1f53978f9cef43dbf7a3d0012220f229ec5a96f4445a12"
        "f4c314b6a789bb99391fecb49c9b91800c7c5c94ccb6012a20a9a2cf11e230952ddbf772551a29786b2d4fef65388b2f3ea22cec9efc0f54013"
        "220dfb0cd1034c290a673a6825cc6771d1c7b97c063b429ff089e890cb85175320c121f120f6b696b7a7262762d373935363239341a0c0a0a08"
        "9ff59eb6b88a838f0b");

    err = parser_parse(&ctx, buffer, bufferLen, &tx_obj);
    ASSERT_EQ(err, parser_ok) << parser_getErrorDescription(err);

    err = parser_computeEffectHash(&ctx);
    ASSERT_EQ(err, parser_ok) << parser_getErrorDescription(err);

    std::string expected =
        "0e9fd733a3724555dd48a8ca91010b8d57b1291ee90784fc8cbfb2c152b1e762065a01b4fa9af5f0ee510ffcc2cb07987b2a4983e9481e6341e"
        "e94ddec213b4f";
    char actual[129];
    array_to_hexstr(actual, sizeof(actual), tx_obj.effect_hash, sizeof(tx_obj.effect_hash));

    EXPECT_EQ(std::string(actual), expected);
}
