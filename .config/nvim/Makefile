.PHONY: nvim nvim-convtools nvim-aprenita

nvim:
	./make_nvim_executable \
		--executable-name nvim-python-3.7.9 \
		--python-version 3.7.9 #--force

nvim-convtools:
	./make_nvim_executable \
		--executable-name nvim-convtools \
		--python-version 3.6.12 \
		--from-venv-path ~/.pyenv/versions/convtools

nvim-aprenita:
	./make_nvim_executable \
		--executable-name nvim-aprenita \
		--from-docker-image aprenita-infrastructure_backend
