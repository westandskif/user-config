DIR := $(realpath ${PWD})
AUTOGEN_SCRIPTS_DIR := ${DIR}/autogen_scripts

.PHONY: nvim
# DEFAULT ONE
nvim:
	./install_nvim \
		--output-executable-path ${AUTOGEN_SCRIPTS_DIR}/nvim-python-3.7.6 \
		--python-version 3.7.6 # --force
	sudo ln -sf ${AUTOGEN_SCRIPTS_DIR}/nvim-python-3.7.6 /usr/local/bin/nvim

nvim-convtools:
	PYTHONPATH=/home/nik/own/convtools/src \
	./make_python_project_nvim_executable \
		--executable-name nvim-convtools \
		--python-version 3.6.10 \
		--from-venv-path ~/.pyenv/versions/convtools

nvim-aprenita:
	./make_python_project_nvim_executable \
		--executable-name nvim-aprenita \
		--from-docker-image aprenita-infrastructure_backend #--force-nvim-reinstall
