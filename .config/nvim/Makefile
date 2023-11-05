.PHONY: nvim nvim-convtools nvim-aprenita

nvim:
	./make_nvim_executable \
		--executable-name nvim-python-3.9.7 \
		--python-from-version 3.9.7 \
		--node-from-version 18
	sudo ln -sf /usr/local/bin/nvim-python-3.9.7 /usr/local/bin/nvim

nvim-convtools:
	./make_nvim_executable \
		--executable-name nvim-convtools \
		--python-from-venv-path ~/.pyenv/versions/convtools-3.9

# nvim-aprenita:
# 	./make_nvim_executable \
# 		--executable-name nvim-aprenita \
# 		--python-from-version 3.9.7 \
# 		--python-from-docker-image aprenita-infrastructure_backend \
# 		--node-from-version v16.8.0 \
# 		--node-from-docker-image aprenita-infrastructure_frontend:ts \
# 		--node-modules-output-dir ~/work/aprenita/ui/node_modules \
# 		--node-from-docker-path /node_modules
# 		# --node-from-docker-path /pkg/node_modules \

nvim-aprenita:
	./make_nvim_executable \
		--executable-name nvim-aprenita \
		--python-from-version 3.9.7 \
		--node-from-version 18 \
		--python-from-docker-image aprenita-infrastructure_backend:master

nvim-p190:
	./make_nvim_executable \
		--executable-name nvim-p190 \
		--python-from-version 3.9.7 \
		--node-from-version 18 \
		--python-from-docker-image p190-backend

nvim-poc-bgv:
	./make_nvim_executable \
		--executable-name nvim-poc-bgv \
		--python-from-version 3.10.9 \
		--python-from-docker-image poc-publishing-backend-backend:latest \
		--node-from-version v16.8.0

nvim-p182:
	./make_nvim_executable \
		--executable-name nvim-p182 \
		--python-from-docker-image project-182-api \
		--x-python-from-docker-image--entrypoint "poetry run python" \
		--node-from-version v18.12.0 \
		--node-from-docker-image project-182-ui \
		--node-modules-output-dir ~/own/project-182/ui/node_modules \
		--node-from-docker-path /context/node_modules

