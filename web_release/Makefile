test : core serve

prepare : core zip

core : build copy

build : 
	cargo build --release --target wasm32-unknown-unknown

copy :
	cp -r ../assets .
	cp ../target/wasm32-unknown-unknown/release/*.wasm ./game.wasm


zip : 
	zip -r game.zip assets game.wasm *.js index.html 

# Handy thing to serve the folder
serve :
	python -m http.server
