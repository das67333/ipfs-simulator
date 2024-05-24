# IPFS Simulator

Основные методы, используемые в сети IPFS - `Peer::PublishData(data)` и `Peer::RetrieveData(key)`. Начальное состояние всех [KBucketsTable](src/kbucket/bucket.rs), [топология сети](src/network/topology.rs), [распределение задержек](src/network/delay_distribution.rs), [фоновая пользовательская нагрузка](src/network/user_load.rs), иные параметры симуляции (см. [config.toml](config.toml)) определяют ход исполнения запросов в сети IPFS.
