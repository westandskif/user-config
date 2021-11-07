.PHONY: nvim nvim-convtools nvim-aprenita

nvim:
	./make_nvim_executable \
		--executable-name nvim-python-3.9.7 \
		--python-from-version 3.9.7 #--force
	sudo ln -sf /usr/local/bin/nvim-python-3.9.7 /usr/local/bin/nvim

nvim-convtools:
	./make_nvim_executable \
		--executable-name nvim-convtools \
		--python-from-venv-path ~/.pyenv/versions/convtools

nvim-aprenita:
	./make_nvim_executable \
		--executable-name nvim-aprenita \
		--python-from-version 3.9.7 \
		--python-from-docker-image aprenita-infrastructure_backend \
		--node-from-version v16.8.0 \
		--node-from-docker-image aprenita-infrastructure_frontend:typescript \
		--node-from-docker-path /pkg/node_modules \
		--node-modules-output-dir ~/work/aprenita/ui/node_modules
