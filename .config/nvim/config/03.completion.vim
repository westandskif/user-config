let g:ncm2#auto_popup = 0
let g:ncm2#total_popup_limit = 15
autocmd BufEnter * call ncm2#enable_for_buffer()
set completeopt=noinsert,menuone,noselect

let g:LanguageClient_diagnosticsEnable = 0
let g:LanguageClient_serverCommands = {
    \ 'rust': ['rust-analyzer'],
    \ 'python': ['pyls'],
    \ }
let g:LanguageClient_loggingFile =  expand('/tmp/nvim-LanguageClient.log')
let g:LanguageClient_serverStderr = expand('/tmp/nvim-LanguageServer.log')

let g:neoformat_run_all_formatters = 1
let g:neoformat_enabled_python = ['isort', 'black']
let g:neomake_python_enabled_makers = ['flake8', 'pylint']
let g:neomake_open_list = 2
let g:neomake_virtualtext_current_error = 0
let g:neomake_highlight_lines = 0
let g:neomake_highlight_columns = 0
let g:neomake_spelling_maker = {
	\ 'exe': 'spelling',
	\ 'args': ['%t'],
	\ 'errorformat': '%f:%l:%c:%m',
	\ }
:call neomake#cmd#disable(g:)

let g:gutentags_ctags_exclude = [
    \  '.git', '.mypy_cache', '.ipynb_checkpoints', '__pycache__',
    \  '*.min.{js,css}', 'build/*',
    \ ]
let g:gutentags_generate_on_new = 0
let g:gutentags_generate_on_missing = 0


let g:neomake_git_diff_config = {
			\ 'py': {'filetype': 'python', 'makers': ['flake8']},
			\}
function! NeomakeGitDiff()
	let ext_patterns = map(keys(g:neomake_git_diff_config), {index, val -> '.'.val.'$'})
    let git_files = systemlist("git diff --name-only --cached --diff-filter=AM | grep '".join(ext_patterns, "|")."'")
	let l:maker_name_to_maker = {}
	for changed_file in git_files
		let ext = fnamemodify(changed_file, ':e')
		let ext_config = g:neomake_git_diff_config[ext]
		let changed_file_filetype = ext_config['filetype']
		let needed_makers = ext_config['makers']
		for maker_name in needed_makers
			if !has_key(maker_name_to_maker, maker_name)
				let l:maker_name_to_maker[maker_name] = deepcopy(neomake#GetMaker(maker_name, changed_file_filetype))
				let l:maker_name_to_maker[maker_name].append_file = 0
			endif
			let maker = l:maker_name_to_maker[maker_name]
			call add(maker.args, changed_file)
		endfor
	endfor
    call neomake#Make({'enabled_makers': values(l:maker_name_to_maker)})
endfunction

function! RefreshSitePackageTags()
    let l:text =<< trim END
    import sys
    bad_endings = (".zip", "dynload")
    for path in sys.path:
        if path and any(path.endswith(ending) for ending in bad_endings):
            continue
        print(path)
    END
    let l:sitepackages = systemlist('python -', l:text)
    for path in l:sitepackages
        if stridx(&tags, path) == -1
            let &tags = &tags.path."/tags,"
        endif
    endfor
    return l:sitepackages
endfunction
autocmd FileType python :call RefreshSitePackageTags()

function! GenerateSitePackageTags()
    let l:sitepackages = RefreshSitePackageTags()
    for path in l:sitepackages
        call system("bash -s", "pushd ".path." && rm -f tags && ctags -R --languages=python --exclude=site-packages --exclude=test && sed -i '/\/\^ /d' tags && popd")
    endfor
endfunction
