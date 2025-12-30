.PHONY: nvim nvim-convtools nvim-aprenita


# nvim-p190:
# 	./make_nvim_executable \
# 		--executable-name nvim-p190 \
# 		--python-from-version 3.9.7 \
# 		--node-from-version 18 \
# 		--python-from-docker-image p190-backend \
# 	    --x-python-from-docker-image--entrypoint "poetry run python"

nvim-convtools: export EXECUTABLE_NAME=nvim-convtools
nvim-convtools:
	VENV_PATH=~/.pyenv/versions/convtools-3.10 \
		./scripts/py_from_venv
	./scripts/nvim_py_tools
	./scripts/generate_tags
	./scripts/nvim_executable

nvim-p262: export EXECUTABLE_NAME=nvim-p262
nvim-p262:
	VENV_PATH=~/.pyenv/versions/p262-py3.13 \
		./scripts/py_from_venv
	./scripts/nvim_py_tools
	./scripts/generate_tags
	./scripts/nvim_executable


nvim-cp: export EXECUTABLE_NAME=nvim-cp
nvim-cp:
	DOCKER_RUN_OPTS="--entrypoint=python aprenita-infrastructure_backend:master" \
		./scripts/py_from_docker
	DOCKER_RUN_OPTS="--entrypoint=bash aprenita-infrastructure_frontend:master" \
		DOCKER_NODE_MODULES_PATH="/node_modules" \
		NODE_MODULES_OUTPUT_DIR="$$HOME/work/aprenita/ui/node_modules" \
		./scripts/js_from_docker
	./scripts/nvim_py_tools
	./scripts/nvim_js_tools
	./scripts/generate_tags
	./scripts/nvim_executable


nvim-vp: export EXECUTABLE_NAME=nvim-vp
nvim-vp:
	VENV_PATH=~/.pyenv/versions/aprenita-virtual-people \
		./scripts/py_from_venv
	./scripts/nvim_py_tools
	./scripts/generate_tags
	./scripts/nvim_executable


nvim-synclane: export EXECUTABLE_NAME=nvim-synclane
nvim-synclane:
	VENV_PATH=~/.pyenv/versions/synclane-3.9 \
		./scripts/py_from_venv
	./scripts/nvim_py_tools
	./scripts/nvim_js_tools
	./scripts/generate_tags
	./scripts/nvim_executable


nvim-pub: export EXECUTABLE_NAME=nvim-pub
nvim-pub:
	DOCKER_RUN_OPTS="--entrypoint=python publishing-backend-backend:ccf" \
		./scripts/py_from_docker
	./scripts/nvim_py_tools
	./scripts/generate_tags
	./scripts/nvim_executable


nvim-p620: export EXECUTABLE_NAME=nvim-p620
nvim-p620:
	DOCKER_RUN_OPTS="--entrypoint=python p620-py_api:latest" \
		./scripts/py_from_docker
	# DOCKER_RUN_OPTS="--entrypoint=bash p620-js_ui:latest" \
	# 	DOCKER_NODE_MODULES_PATH="/node_modules" \
	# 	NODE_MODULES_OUTPUT_DIR="$$HOME/own/p620/ui/node_modules" \
	#  	./scripts/js_from_docker
	# ./scripts/nvim_py_tools
	# ./scripts/nvim_js_tools
	# ./scripts/generate_tags
	# ./scripts/nvim_executable
