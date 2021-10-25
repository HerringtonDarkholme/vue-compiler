const { compile_sync_buffer } = require('./index')

// console.assert(sync(0) === 100, 'Simple test failed')

console.info('Simple test passed')

console.log(compile_sync_buffer(Buffer.from('hello world')))
