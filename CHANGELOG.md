# Changelog

## 0.1.0 (2026-03-17)


### Features

* add --latest flag and held-back messaging to update ([f328e00](https://github.com/JamesGuthrie/unpm/commit/f328e0080b91d25724de96a49261c08760d03b85))
* add canonical mode ([635abfd](https://github.com/JamesGuthrie/unpm/commit/635abfd1746ebe85359bab9a278ba44c587690cd))
* add CLI structure with clap ([2dd9adc](https://github.com/JamesGuthrie/unpm/commit/2dd9adcca0028c309da5c53c5cb98b32bdeb22c0))
* add config, manifest, and lockfile data types ([9c0f2f5](https://github.com/JamesGuthrie/unpm/commit/9c0f2f54d3941fc1a88dad5d7e0c2a2064e27401))
* add CVE checking via OSV.dev API ([3c56d7d](https://github.com/JamesGuthrie/unpm/commit/3c56d7d19317454edad369eb527ad2b9b994c692))
* add GitHub-hosted package support via gh: prefix ([f110574](https://github.com/JamesGuthrie/unpm/commit/f110574d27b5a5c3a97d8e1cfda0d5d1538d9be6))
* add HTTP fetcher with SHA-256 verification ([5e96f60](https://github.com/JamesGuthrie/unpm/commit/5e96f60720eb640735b68193e381ee2ce78d159c))
* add jsdelivr registry client ([13cf003](https://github.com/JamesGuthrie/unpm/commit/13cf003ecc6eeb193037156e1e1ca57fc10e7093))
* add list command ([902a6b1](https://github.com/JamesGuthrie/unpm/commit/902a6b1dc9827d50dbe23e7f368fdfd17f7d4590))
* add outdated command ([9e550a8](https://github.com/JamesGuthrie/unpm/commit/9e550a8ab72685fc4552c0bb5a2f07fbe5576851))
* add outdated reporting to check ([7954f12](https://github.com/JamesGuthrie/unpm/commit/7954f12b204fde43df9e3f4e0ccac57ad32f1a02))
* add update command ([1519400](https://github.com/JamesGuthrie/unpm/commit/1519400423019f94105ed3b0675df22d535b297e))
* add vendor module for file placement ([1eb1c5e](https://github.com/JamesGuthrie/unpm/commit/1eb1c5e518fdc9382de13f251d8c7f4f58e73f52))
* implement check command ([9c3ca9e](https://github.com/JamesGuthrie/unpm/commit/9c3ca9e21d0e21bc2b10e56b63c498f4b50a6e71))
* implement install command with SHA verification ([256c499](https://github.com/JamesGuthrie/unpm/commit/256c499e0c6751a058a3281a2a93ed4f9bf7a949))
* implement interactive add command ([9781670](https://github.com/JamesGuthrie/unpm/commit/9781670920ad834baea343eeb636b8418aaf30ca))
* implement remove command ([c9e9d47](https://github.com/JamesGuthrie/unpm/commit/c9e9d47cec9126e3ac88e7138c412e199c8fe4c6))
* resolve all GitHub versions to commit SHAs via GitHub API ([23a1001](https://github.com/JamesGuthrie/unpm/commit/23a10012b9c5cf5c46c298cb9f1cec97dfef0692))
* same-major update by default, support update-all ([cae92dc](https://github.com/JamesGuthrie/unpm/commit/cae92dc0610cfa26b46cb9be2a6a5027404b55b2))
* skip minification prompt when user explicitly selects a file ([786dcd1](https://github.com/JamesGuthrie/unpm/commit/786dcd1194a23a12c0ce9f8d65c770b6eab9b602))
* support multiple files per dependency ([#2](https://github.com/JamesGuthrie/unpm/issues/2)) ([9c97c3d](https://github.com/JamesGuthrie/unpm/commit/9c97c3d871e23ae00aa53e0f49f1219f37f66bff))
* support package@version syntax in add command ([db7e794](https://github.com/JamesGuthrie/unpm/commit/db7e794f2817efe2ebf55e1d98a535b24da72f3d))
* verify release binary attestations in GitHub Action ([7abd036](https://github.com/JamesGuthrie/unpm/commit/7abd0360fe54e9dc5ff387265bf439c7716ea975))


### Bug Fixes

* add debug logging ([4e0decc](https://github.com/JamesGuthrie/unpm/commit/4e0deccad1d68ed5a97b37cd13c464725163f008))
* correct repo owner in URLs and action references ([e2b93a9](https://github.com/JamesGuthrie/unpm/commit/e2b93a96422d46c618d3131aef4f62a622f2af5a))
* only namespace vendored filenames on collision ([4278e5f](https://github.com/JamesGuthrie/unpm/commit/4278e5f1bfbef0ec91c8e17dc27614bbcf96ddb1))
* prevent path traversal via lockfile filename ([31f12e9](https://github.com/JamesGuthrie/unpm/commit/31f12e98cd68dafb01e413f937b48d130c22b9a8))
* prevent shell injection in GitHub Action inputs ([72a79dc](https://github.com/JamesGuthrie/unpm/commit/72a79dc87424c2258b2f13bebe538852af0cdd92))
* reject HTTP responses larger than 50 MB ([aefdb8d](https://github.com/JamesGuthrie/unpm/commit/aefdb8d59309f900c6d3ac432a5dc34862761c14))
* report CDN hash decode errors instead of silently mismatching ([64cabc7](https://github.com/JamesGuthrie/unpm/commit/64cabc74113cc322e9900352cba3210daec288ec))
* resolve clippy warnings ([250afc9](https://github.com/JamesGuthrie/unpm/commit/250afc99d928c61e745968426a21f2ee55c2224f))
* resolve default path when jsdelivr returns a non-existent minified file ([feb7ac5](https://github.com/JamesGuthrie/unpm/commit/feb7ac5cda736373bcdaa3b4d4d274845a314d0f))
* reverse list output order ([9e62a5f](https://github.com/JamesGuthrie/unpm/commit/9e62a5f70c9010dc7c4eb618bb0017b3671968fe))
* security, shared client, parallel checks ([2238ff8](https://github.com/JamesGuthrie/unpm/commit/2238ff84e746a2e52462bf59997407e548afb5bf))
* set release-please manifest to 0.0.0 so first release targets 0.1.0 ([6b2894c](https://github.com/JamesGuthrie/unpm/commit/6b2894c8469ba4065450ebafd864f80402927a70))
* use lockfile version for CDN hash verification of GitHub deps ([09e76a2](https://github.com/JamesGuthrie/unpm/commit/09e76a2799f7806ff8953217bbc0604fefb3764a))
* version ordering, jsdelivr returns newest-first ([56ffe8b](https://github.com/JamesGuthrie/unpm/commit/56ffe8bf578e351124dc0e2df69d4447ec2c1f29))

## [0.2.0](https://github.com/JamesGuthrie/unpm/compare/unpm-v0.1.0...unpm-v0.2.0) (2026-03-16)


### Features

* resolve all GitHub versions to commit SHAs via GitHub API ([23a1001](https://github.com/JamesGuthrie/unpm/commit/23a10012b9c5cf5c46c298cb9f1cec97dfef0692))


### Bug Fixes

* use lockfile version for CDN hash verification of GitHub deps ([09e76a2](https://github.com/JamesGuthrie/unpm/commit/09e76a2799f7806ff8953217bbc0604fefb3764a))

## 0.1.0 (2026-03-16)


### Features

* add --latest flag and held-back messaging to update ([f328e00](https://github.com/JamesGuthrie/unpm/commit/f328e0080b91d25724de96a49261c08760d03b85))
* add canonical mode ([635abfd](https://github.com/JamesGuthrie/unpm/commit/635abfd1746ebe85359bab9a278ba44c587690cd))
* add CLI structure with clap ([2dd9adc](https://github.com/JamesGuthrie/unpm/commit/2dd9adcca0028c309da5c53c5cb98b32bdeb22c0))
* add config, manifest, and lockfile data types ([9c0f2f5](https://github.com/JamesGuthrie/unpm/commit/9c0f2f54d3941fc1a88dad5d7e0c2a2064e27401))
* add CVE checking via OSV.dev API ([3c56d7d](https://github.com/JamesGuthrie/unpm/commit/3c56d7d19317454edad369eb527ad2b9b994c692))
* add GitHub-hosted package support via gh: prefix ([f110574](https://github.com/JamesGuthrie/unpm/commit/f110574d27b5a5c3a97d8e1cfda0d5d1538d9be6))
* add HTTP fetcher with SHA-256 verification ([5e96f60](https://github.com/JamesGuthrie/unpm/commit/5e96f60720eb640735b68193e381ee2ce78d159c))
* add jsdelivr registry client ([13cf003](https://github.com/JamesGuthrie/unpm/commit/13cf003ecc6eeb193037156e1e1ca57fc10e7093))
* add list command ([902a6b1](https://github.com/JamesGuthrie/unpm/commit/902a6b1dc9827d50dbe23e7f368fdfd17f7d4590))
* add outdated command ([9e550a8](https://github.com/JamesGuthrie/unpm/commit/9e550a8ab72685fc4552c0bb5a2f07fbe5576851))
* add outdated reporting to check ([7954f12](https://github.com/JamesGuthrie/unpm/commit/7954f12b204fde43df9e3f4e0ccac57ad32f1a02))
* add update command ([1519400](https://github.com/JamesGuthrie/unpm/commit/1519400423019f94105ed3b0675df22d535b297e))
* add vendor module for file placement ([1eb1c5e](https://github.com/JamesGuthrie/unpm/commit/1eb1c5e518fdc9382de13f251d8c7f4f58e73f52))
* implement check command ([9c3ca9e](https://github.com/JamesGuthrie/unpm/commit/9c3ca9e21d0e21bc2b10e56b63c498f4b50a6e71))
* implement install command with SHA verification ([256c499](https://github.com/JamesGuthrie/unpm/commit/256c499e0c6751a058a3281a2a93ed4f9bf7a949))
* implement interactive add command ([9781670](https://github.com/JamesGuthrie/unpm/commit/9781670920ad834baea343eeb636b8418aaf30ca))
* implement remove command ([c9e9d47](https://github.com/JamesGuthrie/unpm/commit/c9e9d47cec9126e3ac88e7138c412e199c8fe4c6))
* same-major update by default, support update-all ([cae92dc](https://github.com/JamesGuthrie/unpm/commit/cae92dc0610cfa26b46cb9be2a6a5027404b55b2))
* skip minification prompt when user explicitly selects a file ([786dcd1](https://github.com/JamesGuthrie/unpm/commit/786dcd1194a23a12c0ce9f8d65c770b6eab9b602))
* support multiple files per dependency ([#2](https://github.com/JamesGuthrie/unpm/issues/2)) ([9c97c3d](https://github.com/JamesGuthrie/unpm/commit/9c97c3d871e23ae00aa53e0f49f1219f37f66bff))
* support package@version syntax in add command ([db7e794](https://github.com/JamesGuthrie/unpm/commit/db7e794f2817efe2ebf55e1d98a535b24da72f3d))
* verify release binary attestations in GitHub Action ([7abd036](https://github.com/JamesGuthrie/unpm/commit/7abd0360fe54e9dc5ff387265bf439c7716ea975))


### Bug Fixes

* add debug logging ([4e0decc](https://github.com/JamesGuthrie/unpm/commit/4e0deccad1d68ed5a97b37cd13c464725163f008))
* correct repo owner in URLs and action references ([e2b93a9](https://github.com/JamesGuthrie/unpm/commit/e2b93a96422d46c618d3131aef4f62a622f2af5a))
* only namespace vendored filenames on collision ([4278e5f](https://github.com/JamesGuthrie/unpm/commit/4278e5f1bfbef0ec91c8e17dc27614bbcf96ddb1))
* prevent path traversal via lockfile filename ([31f12e9](https://github.com/JamesGuthrie/unpm/commit/31f12e98cd68dafb01e413f937b48d130c22b9a8))
* prevent shell injection in GitHub Action inputs ([72a79dc](https://github.com/JamesGuthrie/unpm/commit/72a79dc87424c2258b2f13bebe538852af0cdd92))
* reject HTTP responses larger than 50 MB ([aefdb8d](https://github.com/JamesGuthrie/unpm/commit/aefdb8d59309f900c6d3ac432a5dc34862761c14))
* report CDN hash decode errors instead of silently mismatching ([64cabc7](https://github.com/JamesGuthrie/unpm/commit/64cabc74113cc322e9900352cba3210daec288ec))
* resolve clippy warnings ([250afc9](https://github.com/JamesGuthrie/unpm/commit/250afc99d928c61e745968426a21f2ee55c2224f))
* resolve default path when jsdelivr returns a non-existent minified file ([feb7ac5](https://github.com/JamesGuthrie/unpm/commit/feb7ac5cda736373bcdaa3b4d4d274845a314d0f))
* reverse list output order ([9e62a5f](https://github.com/JamesGuthrie/unpm/commit/9e62a5f70c9010dc7c4eb618bb0017b3671968fe))
* security, shared client, parallel checks ([2238ff8](https://github.com/JamesGuthrie/unpm/commit/2238ff84e746a2e52462bf59997407e548afb5bf))
* set release-please manifest to 0.0.0 so first release targets 0.1.0 ([6b2894c](https://github.com/JamesGuthrie/unpm/commit/6b2894c8469ba4065450ebafd864f80402927a70))
* version ordering, jsdelivr returns newest-first ([56ffe8b](https://github.com/JamesGuthrie/unpm/commit/56ffe8bf578e351124dc0e2df69d4447ec2c1f29))
