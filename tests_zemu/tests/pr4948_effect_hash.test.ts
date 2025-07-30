/** ******************************************************************************
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
 ******************************************************************************* */

import Zemu from '@zondax/zemu'
import { PENUMBRA_PATH, defaultOptions, models, ACCOUNT_ID } from './common'
import { PenumbraApp, AddressIndex } from '@zondax/ledger-penumbra'
import { PR4948TestCase } from './testscases/types'
import * as fs from 'fs'
import * as path from 'path'

const PR4948_TESTCASES: PR4948TestCase[] = JSON.parse(
  fs.readFileSync(path.join(__dirname, '../penumbra_pr4948_action_testcases.json'), 'utf8')
)


jest.setTimeout(60000)

  describe.each(PR4948_TESTCASES)('PR4948 Effect Hash', function (data) {
    test.concurrent.each(models.filter(m => m.name === 'nanosp'))(`sign ${data.actionTypes.join()}`, async function (m) {
      const sim = new Zemu(m.path)
      try {
        await sim.start({ ...defaultOptions, model: m.name })
        const app = new PenumbraApp(sim.getTransport())

        const messageToSign = Buffer.from(data.blob, 'hex')

        const addressIndex: AddressIndex = {
          account: ACCOUNT_ID,
          randomizer: undefined,
        }

        let signatureResponse = undefined; 
        try {
        // do not wait here... we need to navigate
        const signatureRequest = app.sign(PENUMBRA_PATH, messageToSign, data.metadata)

        // Wait until we are not in the main menu
        await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
        await sim.waitForText('Review')
        await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-sign_${data.name}`)

        signatureResponse = await signatureRequest
        console.log('Effect hash:', signatureResponse.effectHash.toString('hex'))
        } catch (error) {
          throw new Error(`Error signing transaction: ${error}`)
        }


        // Verify effect hash matches exactly (deterministic)
        expect(signatureResponse.effectHash.toString('hex')).toEqual(data.expected_effect_hash)

        // Verify exact signature counts match expected array lengths
        expect(signatureResponse.spendAuthSignatures.length).toEqual(data.expected_spend_sigs.length)
        expect(signatureResponse.delegatorVoteSignatures.length).toEqual(data.expected_delegator_vote_sigs.length)
        expect((signatureResponse as {lqtVoteSignatures?: []}).lqtVoteSignatures?.length ?? 0).toEqual(data.expected_lqt_vote_sigs.length)

      } finally {
        await sim.close()
      }
    })
  })