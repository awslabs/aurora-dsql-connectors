/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/*
 * This code is based on postgres.js community test
 * License: Unlicense (https://github.com/porsager/postgres/blob/5c8135f3df1bb10e7aad10f14a6f084db3724f82/UNLICENSE)
 * Source: https://github.com/porsager/postgres/blob/5c8135f3df1bb10e7aad10f14a6f084db3724f82/tests/test.js
 */

/* eslint no-console: 0 */

import util from 'util'

let done = 0
let only = false
let ignored = 0
let failed = false
let promise = Promise.resolve()
const tests = {}
  , ignore = {}

export const nt = () => ignored++
export const ot = (...rest) => (only = true, test(true, ...rest))
export const t = (...rest) => test(false, ...rest)
t.timeout = 60

async function test(o, name, options, fn) {
  typeof options !== 'object' && (fn = options, options = {})
  const line = new Error().stack.split('\n')[3].match(':([0-9]+):')[1]

  await 1

  if (only && !o)
    return

  tests[line] = { fn, line, name }
  promise = promise.then(() => Promise.race([
    new Promise((resolve, reject) =>
      fn.timer = setTimeout(() => reject('Timed out'), (options.timeout || t.timeout) * 1000)
    ),
    failed
      ? (ignored++, ignore)
      : fn()
  ]))
    .then(async x => {
      clearTimeout(fn.timer)
      if (x === ignore)
        return

      if (!Array.isArray(x))
        throw new Error('Test should return result array')

      const [expected, got] = await Promise.all(x)
      if (expected !== got) {
        failed = true
        throw new Error(util.inspect(expected) + ' != ' + util.inspect(got))
      }

      tests[line].succeeded = true
      process.stdout.write(`✅ `)
      // modified - display test case name and added delay to avoid OC001 errors between test cases
      console.log(name);
      await new Promise(r => setTimeout(r, 500))
    })
    .catch(err => {
      tests[line].failed = failed = true
      // modified - display failure, name and error
      process.stdout.write('❌ ')
      console.log(name);
      tests[line].error = err instanceof Error ? err : new Error(util.inspect(err))
      console.log(err);
    })
    .then(() => {
      ++done === Object.keys(tests).length && exit()
    })
}

function exit() {
  let success = true
  Object.values(tests).every((x) => {
    if (x.succeeded)
      return true

    success = false
    x.cleanup
      ? console.error('⛔️', x.name + ' at line', x.line, 'cleanup failed', '\n', util.inspect(x.cleanup))
      : console.error('⛔️', x.name + ' at line', x.line, x.failed
        ? 'failed'
        : 'never finished', x.error ? '\n' + util.inspect(x.error) : ''
      )
  })

  only
    ? console.error('⚠️', 'Not all tests were run')
    : ignored
      ? console.error('⚠️', ignored, 'ignored test' + (ignored === 1 ? '' : 's', '\n'))
      : success
        ? console.log('🎉')
        : console.error('⚠️', 'Not good')

  // modified - removed ignored in the following code as we have nt() test cases, 
  // the exit code will be always 1 if we have ignored test cases
  !process.exitCode && (!success || only) && (process.exitCode = 1)
}
