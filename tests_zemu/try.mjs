import TransportNodeHid from '@ledgerhq/hw-transport-node-hid'
import { PenumbraApp } from '@zondax/ledger-penumbra'


async function main() {
  const transport = await TransportNodeHid.default.open()

  const app = new PenumbraApp(transport)

  const PEN_PATH = "m/44'/6532'"
  const ACCOUNT_ID = 1
  const RANDOMIZER = '770187941264c925f8ba8776'
  const PLAN = '0a9102128e020a300a0a08d5f28fccf7b7d5bd0312220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a1012520a507484aa099c5fd08c1d725cbfc1acff5b000a33504d77fd8073ae75d9888d7c51ff623eec0063b48bc2b6c0ac20e76c1b9f439c4f22f7b935fd88b7ee4d58afb949f655a4f00e3f65bddfa02912860b9c1a201194995744b0ef87acd41a5dc7473e82a5291c958901a4face985c129b67325a22208169a1a5a29f5dd8dff77e1076daf5191870f526f5da11f4d2f8d0060b2138042a20b47e7cb3e22195693363f8f98f97e9775cf7db9e5ecb3d83d2b5c61ea58bc010322097af9d19059c543d212507fa7cfb3a6b92dbeb6f199934e3daadddef33fe0b06125108a3b7e7fb19123b6775676d7375776d7866676279766d7471766f716a786571766963636d2d36313935303834363639353133383038393038313434383637393436381a0c0a0a08abc3849395f5a9fb0c'

  
  let resp = await app.deviceInfo()
  console.log('Device Info', resp)
  resp = await app.getVersion()
  console.log('Version', resp)

  resp = await app.showAddress(PEN_PATH, ACCOUNT_ID, RANDOMIZER)
  console.log(resp)
  const address = resp.address
  console.log(address.toString('hex'))

  const messageToSign = Buffer.from(PLAN, 'hex')
  try {
    const signatureRequest = app.sign(PEN_PATH, 0, messageToSign)
    const signatureResponse = await signatureRequest
    console.log(signatureResponse.signature.toString('hex'))
  } catch (e) {
    console.log(e)
  }


}

;(async () => {
  await main()
})()
