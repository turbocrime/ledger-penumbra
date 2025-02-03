/*******************************************************************************
 *   (c) 2018 - 2024 Zondax AG
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

#include <hexutils.h>
#include <json/json.h>
#include <parser_txdef.h>

#include <fstream>
#include <iostream>

#include "app_mode.h"
#include "crypto.h"
#include "gmock/gmock.h"
#include "parser.h"
#include "parser_interface.h"
#include "utils/common.h"
#include "zxformat.h"

using ::testing::TestWithParam;

typedef struct {
    uint64_t index;
    std::string name;
    std::string blob;
    std::string hash;
} testcase_effect_hash_t;

class JsonTestsEffectHash : public ::testing::TestWithParam<testcase_effect_hash_t> {
   public:
    struct PrintToStringParamName {
        template <class ParamType>
        std::string operator()(const testing::TestParamInfo<ParamType> &info) const {
            auto p = static_cast<testcase_effect_hash_t>(info.param);
            std::stringstream ss;
            ss << p.index << "_" << p.name;
            return ss.str();
        }
    };
};

// Retrieve testcases from json file
std::vector<testcase_effect_hash_t> GetJsonTestCasesEffectHash(std::string jsonFile) {
    auto answer = std::vector<testcase_effect_hash_t>();

    Json::CharReaderBuilder builder;
    Json::Value obj;

    std::string fullPathJsonFile = std::string(TESTVECTORS_DIR) + jsonFile;

    std::ifstream inFile(fullPathJsonFile);
    if (!inFile.is_open()) {
        return answer;
    }

    // Retrieve all test cases
    JSONCPP_STRING errs;
    Json::parseFromStream(builder, inFile, &obj, &errs);
    std::cout << "Number of testcases: " << obj.size() << std::endl;

    for (int i = 0; i < obj.size(); i++) {
        answer.push_back(testcase_effect_hash_t{obj[i]["index"].asUInt64(), obj[i]["name"].asString(),
                                                obj[i]["blob"].asString(), obj[i]["hash"].asString()});
    }

    return answer;
}

void check_testcase_effect_hash(const testcase_effect_hash_t &tc, bool expert_mode) {
    app_mode_set_expert(expert_mode);

    parser_context_t ctx = {0};
    parser_error_t err;
    zxerr_t zxerr;

    uint8_t buffer[5000];
    uint16_t bufferLen = parseHexString(buffer, sizeof(buffer), tc.blob.c_str());

    parser_tx_t tx_obj = {0};

    err = parser_parse(&ctx, buffer, bufferLen, &tx_obj);
    ASSERT_EQ(err, parser_ok) << parser_getErrorDescription(err);

    err = parser_computeEffectHash(&ctx);
    ASSERT_EQ(err, parser_ok) << parser_getErrorDescription(err);

    std::string expected = tc.hash;
    char actual[129];
    array_to_hexstr(actual, sizeof(actual), tx_obj.effect_hash, sizeof(tx_obj.effect_hash));

    EXPECT_EQ(std::string(actual), expected);
}

INSTANTIATE_TEST_SUITE_P

    (JsonTestCasesCurrentTxEffectHash, JsonTestsEffectHash,
     ::testing::ValuesIn(GetJsonTestCasesEffectHash("plan_effect_hash_testcases.json")));

TEST_P(JsonTestsEffectHash, CheckUIOutput_CurrentTX) { check_testcase_effect_hash(GetParam(), false); }
