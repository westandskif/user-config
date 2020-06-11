DIR := $(realpath ${PWD})
AUTOGEN_SCRIPTS_DIR := ${DIR}/autogen_scripts

.PHONY: nvim
# DEFAULT ONE
nvim:
	./install_nvim \
		--output-executable-path ${AUTOGEN_SCRIPTS_DIR}/nvim-python-3.7.6 \
		--python-version 3.7.6 # --force
	sudo ln -sf ${AUTOGEN_SCRIPTS_DIR}/nvim-python-3.7.6 /usr/local/bin/nvim

