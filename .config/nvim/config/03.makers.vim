let g:neoformat_run_all_formatters = 1
let g:neoformat_enabled_python = ['isort', 'black']
let g:neoformat_hcl_hclfmt = {'exe': 'hclfmt'}
let g:neoformat_enabled_hcl = ['hclfmt']

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
let g:neomake_python_ruff_maker = {
    \ 'exe': 'ruff',
    \ 'args': ['check', '--quiet', '--no-fix'],
    \ 'errorformat': '%f:%l:%c: %m',
    \ }
:call neomake#cmd#disable(g:)

let g:gutentags_generate_on_new = 0
let g:gutentags_generate_on_missing = 0


let g:neomake_qf_lint_quick = {
			\ 'py': {'filetype': 'python', 'makers': ['flake8']},
			\}
let g:neomake_qf_lint_full = {
			\ 'py': {'filetype': 'python', 'makers': ['pylint']},
			\}
function! NeomakeQf(lint_config)
	let maker_name_to_maker = {}
    let buffers = {}
    for item in getqflist()
        let key = string(item['bufnr'])
        if !has_key(buffers, key)
            let buf_info = getbufinfo(str2nr(key))[0]
            let file_name = buf_info['name']

            let ext = fnamemodify(file_name, ':e')
            if !has_key(a:lint_config, ext)
                continue
            endif
            let ext_config = a:lint_config[ext]
            let file_filetype = ext_config['filetype']
            let needed_makers = ext_config['makers']

            for maker_name in needed_makers
                if !has_key(maker_name_to_maker, maker_name)
                    let maker_name_to_maker[maker_name] = deepcopy(neomake#GetMaker(maker_name, file_filetype))
                    let maker_name_to_maker[maker_name].append_file = 0
                endif
                let maker = maker_name_to_maker[maker_name]
                call add(maker.args, file_name)
            endfor
        endif
        let buffers[key] = ''
    endfor
    call neomake#Make({'enabled_makers': values(maker_name_to_maker)})
endfunction
