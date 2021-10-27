const fs = require('fs')
const path = require('path')
const { compileSyncBuffer } = require('./index')
const { baseCompile } = require('@vue/compiler-core')
const file = `
<template>test</template>
`
const SfcFileLarge = fs.readFileSync(path.resolve(__dirname, '../benches/fixtures/ElTable.vue')).toString()
console.log(baseCompile(SfcFileLarge))
// console.assert(sync(0) === 100, 'Simple test failed')

// console.info('Simple test passed')

// console.log(compileSyncBuffer(Buffer.from('hello world')))
