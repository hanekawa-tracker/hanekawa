# hanekawa

A BitTorrent tracker.

hanekawa doesn't know everything, she just knows what she knows.

## Features

- High performance, comprehensively tested, and round-trip fuzzed `bencode` parser and encoder
- Serde serializer for `bencode` structures
- Benchmark suite for `bencode` parser and encoder
- Implements several tracker-related [BEPs](https://www.bittorrent.org/beps/bep_0000.html)
- Supports both HTTP and UDP tracking

## Implemented BitTorrent Enhancement Proposals
- [x] [BEP 3: The BitTorrent Protocol Specification](https://www.bittorrent.org/beps/bep_0003.html)
- [x] [BEP 7: IPv6 Tracker Extension](https://www.bittorrent.org/beps/bep_0007.html)
- [x] [BEP 15: UDP Tracker Protocol](https://www.bittorrent.org/beps/bep_0015.html)
- [x] [BEP 23: Tracker Returns Compact Peer Lists](https://www.bittorrent.org/beps/bep_0023.html)
- [x] [BEP 41: UDP Tracker Protocol Extensions](https://www.bittorrent.org/beps/bep_0041.html)
- [x] [BEP 48: Tracker Protocol Extension: Scrape](https://www.bittorrent.org/beps/bep_0048.html)

## License

hanekawa is licensed under the GPLv3 license.

Note: files in the `benches/samples` directory are not part of this project.
They are from the [Internet Archive](https://archive.org/) and are **not** covered by this project's license. 
