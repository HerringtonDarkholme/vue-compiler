import fs from 'fs'
import path from 'path'
import Benchmark from 'benchmark'
import glob from 'glob'
import {baseCompile} from '@vue/compiler-core'

let suite = new Benchmark.Suite('Original')
const vueFiles = glob.sync('./fixtures/*.vue').map(vue => {
    const fileName = path.basename(vue)
    const content = fs.readFileSync(vue, {encoding: 'utf8'})
    return [fileName, content] as const
})

for (let [file, content] of vueFiles) {
    suite = suite.add(file, () => {
        baseCompile(content, {
            isNativeTag: t => t !== 'draggable-header-view' && t !== 'tree-item',
            ssr: true,
        })
    })
}

// from https://github.com/rhysd/github-action-benchmark/
function getHumanReadableUnitValue(seconds: number): [number, string] {
    if (seconds < 1.0e-6) {
        return [seconds * 1e9, 'ns'];
    } else if (seconds < 1.0e-3) {
        return [seconds * 1e6, 'us'];
    } else if (seconds < 1.0) {
        return [seconds * 1e3, 'ms'];
    } else {
        return [seconds, 's'];
    }
}
suite
    .on('cycle', (event: Benchmark.Event) => {
        const bench = event.target
        if (process.env.CI) {
            console.log(String(event.target))
        } else {
            const [val, unit] = getHumanReadableUnitValue(bench.stats.mean)
            const time =  `Time: ${val.toFixed(2) + unit}`
            console.log(String(event.target), time)
        }
    })
    .run()
