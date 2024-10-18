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
import { ACCOUNT_ID, PEN_PATH, defaultOptions, models, txBlobExample } from './common'
import { PenumbraApp } from '@zondax/ledger-penumbra'

jest.setTimeout(60000)

const EXPECTED_FVK =
  '3cd58bbb8725bfe4566504b04d7a31b67bb67fd5d09a28364ac7ac2c2fd8710e4fb0d8c51486fc24938ca96564842a84201d266c92b72761e4e99a16b3405103'
const EXPECTED_ADDRESS =
  'e0783360338067fc2ba548f460b3f06f33d3e756ebefa8a8c08c5e12a1e667df228df0720fb9bd963894183bc447e1c7ef591fa9625d4a66b7703eec2ec1ef543454673bb61a4f2a3d861114d6891d69'

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

  test.concurrent.each(models)('get app version', async function (m) {
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

  test.concurrent.each(models)('getFvk', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name })
      const app = new PenumbraApp(sim.getTransport())

      //Define HDPATH
      const resp = await app.getFVK(PEN_PATH, ACCOUNT_ID)

      console.log(resp)

      const fvk = resp.fvk !== undefined ? resp.fvk.toString('hex') : ''

      expect(fvk).toEqual(EXPECTED_FVK)
    } finally {
      await sim.close()
    }
  })

  test.concurrent.each(models)('getAddress', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name })
      const app = new PenumbraApp(sim.getTransport())

      //Define HDPATH
      const resp = await app.getAddress(PEN_PATH, ACCOUNT_ID)

      console.log(resp)

      const address = resp.address !== undefined ? resp.address.toString('hex') : ''

      expect(address).toEqual(EXPECTED_ADDRESS)
    } finally {
      await sim.close()
    }
  })

  test.concurrent.each(models)('getAddressRandomized', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name })
      const app = new PenumbraApp(sim.getTransport())

      //Define HDPATH
      const resp = await app.getAddress(PEN_PATH, ACCOUNT_ID, RANDOMIZER)

      console.log(resp)

      const address = resp.address !== undefined ? resp.address.toString('hex') : ''

      expect(address).not.toEqual(EXPECTED_ADDRESS)
    } finally {
      await sim.close()
    }
  })

  test.concurrent.each(models)('showAddress', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({
        ...defaultOptions,
        model: m.name,
        approveKeyword: isTouchDevice(m.name) ? 'Confirm' : '',
        approveAction: ButtonKind.DynamicTapButton,
      })

      const app = new PenumbraApp(sim.getTransport())

      //Define HDPATH
      const resp = app.showAddress(PEN_PATH, ACCOUNT_ID)
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

      //Define HDPATH
      const resp = app.showAddress(PEN_PATH, ACCOUNT_ID, RANDOMIZER)
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
