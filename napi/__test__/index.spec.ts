import test from 'ava'

import { compileSync, compileSyncBuffer } from '../index'

test('sync function from native code', (t) => {
  const fixture = "hello world"
  const expected = `
return function render(_ctx, _cache) {
  with (_ctx) {
    return "hello world"
  }
}`
  t.is(compileSync(fixture), expected)
})

test('sync function accepting buffer', t => {
  const fixture = Buffer.from("hello world")
  const expected = `
return function render(_ctx, _cache) {
  with (_ctx) {
    return "hello world"
  }
}`
  t.is(compileSyncBuffer(fixture), expected)
})

// test('sleep function from native code', async (t) => {
//   const timeToSleep = 200
//   const value = await sleep(timeToSleep)
//   t.is(value, timeToSleep * 2)
// })
