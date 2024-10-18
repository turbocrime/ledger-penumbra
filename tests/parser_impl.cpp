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

// #{TODO} --> Apply tests that check this app's encoding/libraries

#include "parser_impl.h"

#include <hexutils.h>

#include <iostream>
#include <vector>

#include "gmock/gmock.h"
#include "parser.h"
#include "parser_txdef.h"

using namespace std;

TEST(SCALE, ReadBytes) {
    parser_context_t ctx;
    parser_tx_t tx_obj;
    parser_error_t err;
    uint8_t buffer[6000];
    auto bufferLen = parseHexString(
        buffer, sizeof(buffer),
        "0aaf020aac020aa1010a290a0308904e12220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a1012207fa50bf"
        "e8946e53b33e8328672a0c6300ad9333c04707475f2d6cc502ec8513a1a520a50e0783360338067fc2ba548f460b3f06f33d3e756ebefa8a8c0"
        "8c5e12a1e667df228df0720fb9bd963894183bc447e1c7ef591fa9625d4a66b7703eec2ec1ef543454673bb61a4f2a3d861114d6891d691a208"
        "d0887fd0d20bd002c336054772d56b8f9704a80ac8da45533e72f155f7145032220d1241d9d9f051c437455998e7c1eff33cd085ba9a9092d85"
        "2d0381278bb6b1012a205e3505d5b6b1b99a8284f054bd650e8d4b64a23f6322c05a0ccee46bb481cb0932208c6abfdd0dd9111868fbed6a30c"
        "9dd5477b123f7aefd15bc73b68185f0a036110ab2020aaf020aa2010a2a0a0408a09c0112220a2029ea9c2f3371f6a487e7e95c247041f4a356"
        "f983eb064e5d2b3bcf322ca96a10122079ce0fe2f6695ebe61c22edb8de45e1290448500e0058156cb4e29f4d7234f091a520a50e0783360338"
        "067fc2ba548f460b3f06f33d3e756ebefa8a8c08c5e12a1e667df228df0720fb9bd963894183bc447e1c7ef591fa9625d4a66b7703eec2ec1ef"
        "543454673bb61a4f2a3d861114d6891d6910011a20c3021eb39eed577b5d609f5436d9128ff6c8bcc13e326a4fcab5c749e2f489002220c05b9"
        "ffb6512d40cc8cf03eb1e2b47f3fcc323a0f2db077362b2300eb77658022a20a45a2fc5a64032c3a68ce4ec9c542b564cfaa9b14d3af386bd14"
        "2a645212540f32208cb5d93be25ba643ee760f56624cd051e53ca0e7047e4a95a5faf82f0636f60d0a8b021288020a2a0a0408b0ea0112220a2"
        "029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a1012520a50e0783360338067fc2ba548f460b3f06f33d3e756eb"
        "efa8a8c08c5e12a1e667df228df0720fb9bd963894183bc447e1c7ef591fa9625d4a66b7703eec2ec1ef543454673bb61a4f2a3d861114d6891"
        "d691a20cde64106162059a6800918036eb3c0e9b2872f56d4902259fba9b52d18dbd7c52220a1417029703aa865b08f19acc96eba15928a2178"
        "956f6c9c538ea5577a03ef032a20df5538e38602a2bd15119182c50d0f1b22fd8deba0a728a8fbbfb7e9b8c6fc0932207dee2a1d6d9c85ba685"
        "60cfa514b73897e77fe0355d92c9cc2bb3b522b5f3d110abc021ab9020ad0010a480a220a20116d0cd6de9349c686f5802f7c98179e70d4898a"
        "38e1cc3df93705c94e941d0c12220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a10120408a08d061a02080"
        "122040a0208032a520a50e0783360338067fc2ba548f460b3f06f33d3e756ebefa8a8c08c5e12a1e667df228df0720fb9bd963894183bc447e1"
        "c7ef591fa9625d4a66b7703eec2ec1ef543454673bb61a4f2a3d861114d6891d69322057798bc1a097c26b47e61a9870ca98eb8cbe48e9cba4d"
        "b86f5d8b67c12086afb1220fbf22b3005fdd84a03bdf0950c851a9c6e0618b0eea1004ca75e53ce3deef1031a205f9ddbd8093a028de6597d03"
        "d785302d8336e20a2148e15ae016bad0536d7e0122200d358f3968b24d5e612504b211cadb44ace485ceb3ad38821aa96b926279e6001213120"
        "d70656e756d6272612d746573741a020a0022f4012a780a520a50e0783360338067fc2ba548f460b3f06f33d3e756ebefa8a8c08c5e12a1e667"
        "df228df0720fb9bd963894183bc447e1c7ef591fa9625d4a66b7703eec2ec1ef543454673bb61a4f2a3d861114d6891d691220361218d216cfe"
        "90f77f54f045ff21b464795517c05057c595fd59e4958e3941718032a780a520a50e0783360338067fc2ba548f460b3f06f33d3e756ebefa8a8"
        "c08c5e12a1e667df228df0720fb9bd963894183bc447e1c7ef591fa9625d4a66b7703eec2ec1ef543454673bb61a4f2a3d861114d6891d69122"
        "013296da8c9dfdf969be7c7bd74e67e80977cd91635eb32038619f62c732dc46a18022a780a540a520a506ece16f387e0b932082cb0cf682359"
        "0fc287d068d6f684a36d1fb19bfd6dce8b22850f535824aeb66cb8c41309e6f5b2d58ff7b651ef4e09a09c7e48d770d190880e1827b47823a1d"
        "01f0c4b438a7b43122018bd5cedd0eb952244a296c1e3fba4f417ebdcc1cfec04cb9441a394316a58bd");

    parser_parse(&ctx, buffer, bufferLen, &tx_obj);
}
