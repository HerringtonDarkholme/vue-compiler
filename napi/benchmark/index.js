const b = require('benny')
const fs = require('fs')
const { baseCompile } = require('@vue/compiler-core')
const path = require('path')
const { compileSync, compileSyncBuffer } = require('../index')
// const { compile_sync_buffer, sync } = require('../index')
const SfcFileLarge = fs.readFileSync(path.resolve(__dirname, 'fixtures/ElTable.vue'))
const SfcFileLargeString = SfcFileLarge.toString('utf-8')

const SfcFileSmall = fs.readFileSync(path.resolve(__dirname, 'fixtures/Attribute.vue'))
const SfcFileSmallString = SfcFileSmall.toString('utf-8')

b.suite(
    'small sfc vue',

    b.add('sync string', () => {
        compileSync(SfcFileSmallString)
    }),

    b.add('sync string buffer', () => {
        compileSyncBuffer(SfcFileSmall)
    }),
    b.add('@vue/compiler-core sync string', () => {
        baseCompile(SfcFileSmallString, { ssr: true,})
    }),
    b.cycle(),
    b.complete(),
    // b.save({ file: 'reduce', version: '1.0.0' }),
    //   b.save({ file: 'reduce', format: 'chart.html' }),
)

b.suite(
    'large sfc vue',

    b.add('sync string', () => {
        compileSync(SfcFileLargeString)
    }),

    b.add('sync string buffer', () => {
        compileSyncBuffer(SfcFileLarge)
    }),

    b.add('@vue/compiler-core sync string', () => {
        baseCompile(SfcFileLargeString)
    }),
    b.cycle(),
    b.complete(),
    // b.save({ file: 'reduce', version: '1.0.0' }),
    //   b.save({ file: 'reduce', format: 'chart.html' }),
)
