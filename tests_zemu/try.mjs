import TransportNodeHid from '@ledgerhq/hw-transport-node-hid'
import { PenumbraApp } from '@zondax/ledger-penumbra'


async function main() {
  const transport = await TransportNodeHid.default.open()

  const app = new PenumbraApp(transport)

  const PEN_PATH = "m/44'/6532'"
  const ACCOUNT_ID = 1
  const RANDOMIZER = '770187941264c925f8ba8776'

  
  let resp = await app.deviceInfo()
  console.log('Device Info', resp)
  resp = await app.getVersion()
  console.log('Version', resp)

  resp = await app.showAddress(PEN_PATH, ACCOUNT_ID, RANDOMIZER)
  console.log(resp)
  const address = resp.address
  console.log(address.toString('hex'))

}

;(async () => {
  await main()
})()
