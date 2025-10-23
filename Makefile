.PHONY: station fg_station client fg_client lint coverage

station:
	cargo run -p station

profile_station:
	@(cd station && test -L songs || ln -s ../songs songs)
	cd station && cargo flamegraph --bin station --features profile,dhat-heap

client:
	cargo run -p client

profile_client:
	cd client && cargo flamegraph --bin client --features profile

lint:
	cargo clippy

coverage:
	cargo tarpaulin
