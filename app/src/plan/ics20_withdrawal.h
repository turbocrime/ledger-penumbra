/*******************************************************************************
 *  (c) 2018 - 2023 Zondax AG
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
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <zxmacros.h>

#include "parser_common.h"
#include "parser_txdef.h"
#include "pb_common.h"
#include "pb_decode.h"
#include "zxtypes.h"

#ifdef __cplusplus
extern "C" {
#endif

parser_error_t decode_ics20_withdrawal_plan(const bytes_t *data, ics20_withdrawal_plan_t *withdrawal);

#ifdef __cplusplus
}
#endif
