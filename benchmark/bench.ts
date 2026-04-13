import { Bench } from 'tinybench'

import { add as nativeAdd } from '../index.js'

function jsAdd(a: number, b: number) {
  return a + b
}

const b = new Bench()

b.add('Native a + 100', () => {
  nativeAdd(10, 100)
})

b.add('JavaScript a + 100', () => {
  jsAdd(10, 100)
})

await b.run()

console.table(b.table())
