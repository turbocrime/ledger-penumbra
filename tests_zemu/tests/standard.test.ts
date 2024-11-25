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

import Zemu, { ButtonKind, zondaxMainmenuNavigation, isTouchDevice } from '@zondax/zemu'
import { ACCOUNT_ID, PENUMBRA_PATH, defaultOptions, models, txBlobExample } from './common'
import { PenumbraApp, AddressIndex } from '@zondax/ledger-penumbra'

jest.setTimeout(60000)

const EXPECTED_FVK =
  '92c3e768d3ecf0f2c4d93d879dbc16226fe8540443a8216d6b093d8684865a063a35ee29cccf93149dfa565ea693aa5cd36dc5cf8adff15081038d31e796580b'
const EXPECTED_ADDRESS =
  'fc48056bc3fa38105dec3bbf85360034324a68f5f9ad7b1b5b6796f97d8279ae15308df6d619d93aab071c2ea360d09dd0f3fa4d3a8a49b9f9b208ee42e491efff1162c3525990477dfc81a681b7d7c1'

const RANDOMIZER = '770187941264c925f8ba8776'

describe('Standard', function () {
  test.concurrent.each(models)('can start and stop container', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name })
    } finally {
      await sim.close()
    }
  })

  test.concurrent.each(models)('main menu', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name })
      const nav = zondaxMainmenuNavigation(m.name, [1, 0, 0, 4, -5])
      await sim.navigateAndCompareSnapshots('.', `${m.prefix.toLowerCase()}-mainmenu`, nav.schedule)
    } finally {
      await sim.close()
    }
  })

  test.concurrent.each(models)('$name get app version', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name })
      const app = new PenumbraApp(sim.getTransport())
      try {
        const resp = await app.getVersion()
        console.log(resp)

        expect(resp).toHaveProperty('testMode')
        expect(resp).toHaveProperty('major')
        expect(resp).toHaveProperty('minor')
        expect(resp).toHaveProperty('patch')
      } catch {
        console.log('getVersion error')
      }
    } finally {
      await sim.close()
    }
  })

  test.concurrent.each(models)('$name getFvk', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name })
      const app = new PenumbraApp(sim.getTransport())

      const addressIndex: AddressIndex = {
        account: ACCOUNT_ID,
        randomizer: undefined,
      }

      //Define HDPATH
      const resp = await app.getFVK(PENUMBRA_PATH, addressIndex)

      console.log(resp)

      const fvk = Buffer.concat([resp.ak, resp.nk]).toString('hex')

      expect(fvk).toEqual(EXPECTED_FVK)
    } finally {
      await sim.close()
    }
  })

  test.concurrent.each(models)('$name getAddress', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name })
      const app = new PenumbraApp(sim.getTransport())

      const addressIndex: AddressIndex = {
        account: ACCOUNT_ID,
        randomizer: undefined,
      }

      //Define HDPATH
      const resp = await app.getAddress(PENUMBRA_PATH, addressIndex)

      console.log(resp)

      const address = resp.address !== undefined ? resp.address.toString('hex') : ''

      expect(address).toEqual(EXPECTED_ADDRESS)
    } finally {
      await sim.close()
    }
  })

  test.concurrent.each(models)('$name getAddressRandomized', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name })
      const app = new PenumbraApp(sim.getTransport())

      const addressIndex: AddressIndex = {
        account: ACCOUNT_ID,
        randomizer: Buffer.from(RANDOMIZER, 'hex'),
      }

      //Define HDPATH
      const resp = await app.getAddress(PENUMBRA_PATH, addressIndex)

      console.log(resp)

      const address = resp.address !== undefined ? resp.address.toString('hex') : ''

      expect(address).not.toEqual(EXPECTED_ADDRESS)
    } finally {
      await sim.close()
    }
  })

  test.concurrent.each(models)('$name showAddress', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({
        ...defaultOptions,
        model: m.name,
        approveKeyword: isTouchDevice(m.name) ? 'Confirm' : '',
        approveAction: ButtonKind.DynamicTapButton,
      })

      const app = new PenumbraApp(sim.getTransport())

      const addressIndex: AddressIndex = {
        account: 1,
        randomizer: undefined,
      }

      //Define HDPATH
      const resp = app.showAddress(PENUMBRA_PATH, addressIndex)
      // Wait until we are not in the main menu
      await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
      await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-show_address`)

      const resp2 = await resp

      console.log(resp2)

      const address = resp2.address !== undefined ? resp2.address.toString('hex') : ''

      expect(address).toEqual(EXPECTED_ADDRESS)
    } finally {
      await sim.close()
    }
  })

  test.concurrent.each(models)('showAddressRandomized', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({
        ...defaultOptions,
        model: m.name,
        approveKeyword: isTouchDevice(m.name) ? 'Confirm' : '',
        approveAction: ButtonKind.DynamicTapButton,
      })

      const app = new PenumbraApp(sim.getTransport())

      const addressIndex: AddressIndex = {
        account: ACCOUNT_ID,
        randomizer: Buffer.from(RANDOMIZER, 'hex'),
      }

      //Define HDPATH
      const resp = app.showAddress(PENUMBRA_PATH, addressIndex)
      // Wait until we are not in the main menu
      await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
      await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-show_address_randomized`)

      const resp2 = await resp

      console.log(resp2)
    } finally {
      await sim.close()
    }
  })

  // TODO: WIP
  // test.concurrent.each(models)('sign', async function (m) {
  //   const sim = new Zemu(m.path)
  //   try {
  //     await sim.start({ ...defaultOptions, model: m.name })
  //     const app = new PenumbraApp(sim.getTransport())

  //     const messageToSign = Buffer.from(txBlobExample, 'hex')
  //     console.log("messageToSignLength!!!!!", messageToSign.length)
  //     console.log("messageToSign!!!!!", messageToSign)
  //     // do not wait here... we need to navigate
  //     const signatureRequest = app.sign(PEN_PATH, ACCOUNT_ID, messageToSign)

  //     // Wait until we are not in the main menu
  //     // await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
  //     // await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-sign`)

  //     const signatureResponse = await signatureRequest
  //     console.log(signatureResponse)

  //     // Now verify the signature
  //     // const valid = ed25519.verify(signatureResponse.signature, messageToSign, pubKey)
  //     // expect(valid).toEqual(true)
  //   } finally {
  //     await sim.close()
  //   }
  // })
})
