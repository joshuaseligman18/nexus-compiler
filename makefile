build:
	# Build with the option to go straight to the browser and not deal with a js bundler
	wasm-pack build --target web

clean:
	# Clean up the target and pkg folders
	cargo clean; \
	if [ -d "pkg" ]; \
	then \
		rm -r pkg; \
	fi

run:
	# Spin up a simple web server so the js can fetch the wasm file
	# https://stackoverflow.com/questions/592620/how-can-i-check-if-a-program-exists-from-a-bash-script
	if ! command -v serve &> /dev/null; \
	then \
		npx serve; \
	else \
		serve; \
	fi