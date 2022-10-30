.PHONY: nvim nvim-convtools nvim-aprenita

nvim:
	./make_nvim_executable \
		--executable-name nvim-python-3.9.7 \
		--python-from-version 3.9.7 #--force
	sudo ln -sf /usr/local/bin/nvim-python-3.9.7 /usr/local/bin/nvim

nvim-convtools:
	./make_nvim_executable \
		--executable-name nvim-convtools \
		--python-from-venv-path ~/.pyenv/versions/convtools-3.9

nvim-aprenita:
	./make_nvim_executable \
		--executable-name nvim-aprenita \
		--python-from-version 3.9.7 \
		--python-from-docker-image aprenita-infrastructure_backend \
		--node-from-version v16.8.0 \
		--node-from-docker-image aprenita-infrastructure_frontend:ts \
		--node-modules-output-dir ~/work/aprenita/ui/node_modules \
		--node-from-docker-path /node_modules
		# --node-from-docker-path /pkg/node_modules \

nvim-p182:
	./make_nvim_executable \
		--executable-name nvim-p182 \
		--python-from-docker-image project-182-api \
		--x-python-from-docker-image--entrypoint "poetry run python"

