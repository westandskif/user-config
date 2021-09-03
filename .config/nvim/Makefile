.PHONY: nvim nvim-convtools nvim-aprenita

nvim:
	./make_nvim_executable \
		--executable-name nvim-python-3.7.9 \
		--python-from-version 3.7.9 #--force
	sudo ln -sf /usr/local/bin/nvim-python-3.7.9 /usr/local/bin/nvim

nvim-convtools:
	./make_nvim_executable \
		--executable-name nvim-convtools \
		--python-from-venv-path ~/.pyenv/versions/convtools

nvim-aprenita:
	./make_nvim_executable \
		--executable-name nvim-aprenita \
		--python-from-docker-image aprenita-infrastructure_backend
