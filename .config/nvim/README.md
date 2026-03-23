# Mise

Use `mise` to set up env, nvim tools and scripts to install deps. Add `mise.toml` file.

e.g. when poetry is needed, create 2 venvs:

```toml
[env]
CURRENT_BRANCH = 'master'

[tools]
python = { version = '3.12', postinstall = '''
pip install virtualenv
mkdir -p .venvs
'''}
node = { version = '24' }

[hooks.enter]
shell = 'zsh'
script = '''
source .venvs/$CURRENT_BRANCH/bin/activate
'''
[hooks.leave]
shell = 'zsh'
script = '''
deactivate
'''

[tasks.pyinstall]
description = "Build the CLI"
run = '''
poetry_version="poetry==1.8.3"
if [ ! -d ".venvs/poetry" ]; then
  echo "INSTALLING POETRY $CURRENT_BRANCH VENV"
  virtualenv .venvs/poetry
  .venvs/poetry/bin/pip install "$poetry_version"
else
  echo "ALREADY INSTALLED: $poetry_version"
fi

if [ ! -d ".venvs/$CURRENT_BRANCH" ]; then
  echo "CREATING VENV: $CURRENT_BRANCH"
  virtualenv .venvs/$CURRENT_BRANCH
  source .venvs/$CURRENT_BRANCH/bin/activate
else
  echo "VENV ALREADY CREATED: $CURRENT_BRANCH"
fi

.venvs/$CURRENT_BRANCH/bin/pip install black ruff isort mypy 'python-lsp-server[all]'
.venvs/poetry/bin/poetry install
'''

[tasks.jsinstall]
run = '''
npm install -g \
  prettier-pnp \
  typescript-language-server \
  typescript-formatter \
  typescript
'''
```
