DIR := $(realpath ${PWD})
AUTOGEN_SCRIPTS_DIR := ${DIR}/autogen_scripts

.PHONY: nvim
# DEFAULT ONE
nvim:
	./make_nvim_executable \
		--output-executable-path ${AUTOGEN_SCRIPTS_DIR}/nvim-python-3.7.9 \
		--python-version 3.7.9 #--force
	sudo ln -sf ${AUTOGEN_SCRIPTS_DIR}/nvim-python-3.7.9 /usr/local/bin/nvim

nvim-convtools:
	./make_nvim_executable \
		--output-executable-path ${AUTOGEN_SCRIPTS_DIR}/nvim-convtools \
		--python-version 3.6.12 \
		--from-venv-path ~/.pyenv/versions/convtools

nvim-aprenita:
	./make_nvim_executable \
		--output-executable-path ${AUTOGEN_SCRIPTS_DIR}/nvim-aprenita \
		--from-docker-image aprenita-infrastructure_backend
