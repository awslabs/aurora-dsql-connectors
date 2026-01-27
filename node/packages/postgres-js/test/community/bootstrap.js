/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/*
 * This code is based on postgres.js community test
 * License: Unlicense (https://github.com/porsager/postgres/blob/5c8135f3df1bb10e7aad10f14a6f084db3724f82/UNLICENSE)
 * Source: https://github.com/porsager/postgres/blob/5c8135f3df1bb10e7aad10f14a6f084db3724f82/tests/bootstrap.js
 */

import { spawnSync } from 'child_process'

/* modified - prevent execution of the following on import
exec('dropdb', ['postgres_js_test'])

exec('psql', ['-c', 'alter system set ssl=on'])
exec('psql', ['-c', 'drop user postgres_js_test'])
exec('psql', ['-c', 'create user postgres_js_test'])
exec('psql', ['-c', 'alter system set password_encryption=md5'])
exec('psql', ['-c', 'select pg_reload_conf()'])
exec('psql', ['-c', 'drop user if exists postgres_js_test_md5'])
exec('psql', ['-c', 'create user postgres_js_test_md5 with password \'postgres_js_test_md5\''])
exec('psql', ['-c', 'alter system set password_encryption=\'scram-sha-256\''])
exec('psql', ['-c', 'select pg_reload_conf()'])
exec('psql', ['-c', 'drop user if exists postgres_js_test_scram'])
exec('psql', ['-c', 'create user postgres_js_test_scram with password \'postgres_js_test_scram\''])

exec('createdb', ['postgres_js_test'])
exec('psql', ['-c', 'grant all on database postgres_js_test to postgres_js_test'])
exec('psql', ['-c', 'alter database postgres_js_test owner to postgres_js_test'])
*/

export function exec(cmd, args) {
  const { stderr } = spawnSync(cmd, args, { stdio: 'pipe', encoding: 'utf8' })
  if (stderr && !stderr.includes('already exists') && !stderr.includes('does not exist') && !stderr.includes('WARNING:'))
    throw stderr
}

async function execAsync(cmd, args) { // eslint-disable-line
  let stderr = ''
  const cp = await spawn(cmd, args, { stdio: 'pipe', encoding: 'utf8' }) // eslint-disable-line
  cp.stderr.on('data', x => stderr += x)
  await new Promise(x => cp.on('exit', x))
  if (stderr && !stderr.includes('already exists') && !stderr.includes('does not exist'))
    throw new Error(stderr)
}
