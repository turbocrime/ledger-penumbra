export interface TestCase {
  idx: number
  name: string
  blob: string
  expected_effect_hash: string
  expected_spend_sig: string
  metadata: string[]
}
