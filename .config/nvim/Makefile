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
	VENV_PATH=~/.pyenv/versions/convtools-3.9 \
		./scripts/py_from_venv
	./scripts/nvim_py_tools
	./scripts/generate_tags
	./scripts/nvim_executable


nvim-cp: export EXECUTABLE_NAME=nvim-cp
nvim-cp:
	DOCKER_RUN_OPTS="--entrypoint=python aprenita-infrastructure_backend:apple_go_manual" \
		./scripts/py_from_docker
	DOCKER_RUN_OPTS="--entrypoint=bash aprenita-infrastructure_frontend:apple_go_manual" \
		DOCKER_NODE_MODULES_PATH="/node_modules" \
		NODE_MODULES_OUTPUT_DIR="$$HOME/work/aprenita/ui/node_modules" \
		./scripts/js_from_docker
	./scripts/nvim_py_tools
	./scripts/nvim_js_tools
	./scripts/generate_tags
	./scripts/nvim_executable

