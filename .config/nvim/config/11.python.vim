function! PathAsImport(path)
    let path = substitute(a:path, '/', '.', 'g')
    let path = substitute(path, '.py$', '', '')
    return 'from '. path . ' import '
endfunction
function! s:do_import(import_path)
    execute 'normal O' . a:import_path
endfunction
function! AutoImport(name)
    " find exact tag
    let tags = taglist('^'.a:name.'$')
    " filter top-level tags
    let tags = filter(tags, "v:val['cmd'][2] != ' '")
    let matched_files = map(tags, "v:val['filename']")
    for tagfile in tagfiles()
        let base = fnamemodify(tagfile, ':p:h')
        let matched_files = map(
                    \   matched_files,
                    \   {i, x -> substitute(x, base . '/', '', '')}
                    \ )
    endfor
    " prepare import strings
    let matched_files = map(matched_files, {i, x -> PathAsImport(x).a:name})
    let l:size = len(matched_files)
    if l:size < 1
        echohl WarningMsg
        echo "[AutoImport] Not found!"
        echohl None
        return
    endif
    if l:size == 1
        call s:do_import(matched_files[0])
        return
    endif
    call fzf#run(fzf#wrap({
    \   'source': matched_files,
    \   'sink': function('s:do_import'),
    \ }))
endfunction

function! CleanQf()
  silent execute "normal! :%v/error/d\<cr>"
  silent execute "normal! :%g/has no 'DoesNotExist' member/d\<cr>"
  silent execute "normal! :%g/has no 'objects' member/d\<cr>"
  silent execute "normal! :%g/action is not callable/d\<cr>"
  silent execute "normal! :%g/has no 'is_valid_password' member/d\<cr>"
  silent execute "normal! :%g/unable to import 'ujson'/d\<cr>"
  silent execute "normal! :%g/no name 'lru' in module 'lru'/d\<cr>"
  silent execute "normal! :%g/No name 'NumericValueOutOfRange' in module 'psycopg2.errors'/d\<cr>"
  silent execute "normal! :%g/Class 'CurrencyExchangeRate' has no '_meta' member/d\<cr>"
  silent execute "normal! :%g/quick_books.*no-member/d\<cr>"
  silent execute "normal! :%g/taxes\\/tasks.*E0213/d\<cr>"
  silent execute "normal! :%g/Instance of 'FactoringOnDemandPaymentRequest' has no 'dest_bank_account_id' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'FactoringOnDemandPaymentRequest' has no 'id' member/d\<cr>"
  silent execute "normal! :%g/core\\/contact_list\\//d\<cr>"
  silent execute "normal! :%g/mca\\/manager.*Instance of 'tuple' has no 'details_format' member/d\<cr>"
  silent execute "normal! :%g/core\\/tasks.*Method 'autoscaling_client' has no 'set_instance_protection' member/d\<cr>"
  silent execute "normal! :%g/serializers\\/companies.*Instance of 'tuple' has no 'msg_type' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'NewSimpleAdAccountDataConverter' has no 'date' member/d\<cr>"
  silent execute "normal! :%g/'AuthorizedPerson' has no 'email_links' member/d\<cr>"
  silent execute "normal! :%g/'AuthorizedPerson' has no 'phone_links' member/d\<cr>"
  silent execute "normal! :%g/'CompanyEmailAlias' has no 'service_alias' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'Company' has no 'id' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'Company' has no 'person' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'Company' has no 'alias' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'Company' has no 'credentials' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'Company' has no 'company_user_hrefs' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'Company' has no 'bank_accounts' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'Company' has no 'braavo_cards' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'Company' has no 'settings' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'Company' has no 'stripe_customer_id' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'CompanyAprenitaEmail' has no/d\<cr>"
  silent execute "normal! :%g/Instance of 'OneToOneField' has no/d\<cr>"
  silent execute "normal! :%g/Instance of 'AppleID' has no 'id' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'AppleID' has no 'person' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'ForeignKey' has no/d\<cr>"
  silent execute "normal! :%g/Instance of 'Resource' has no 'buckets' member/d\<cr>"
  silent execute "normal! :%g/Instance of 'IMGroup' has no 'users' member/d\<cr>"
  silent execute "normal! :%g/data_providers\\/analytics.*Raising NoneType while only classes/d\<cr>"
  silent execute "normal! :%g/core\\/tasks.*Method should have \"self\" as first argument/d\<cr>"
  silent execute "normal! :%g/itunes\\/data_providers\\/base.*Raising NoneType while only classes/d\<cr>"
endfunction

function PythonSpecifics()
  nnoremap <buffer> <localleader>lff :Neoformat black<CR>
  nnoremap <buffer> <localleader>lfi :Neoformat isort<CR>
  nnoremap <buffer> <localleader>lmq :Neomake flake8<CR>
  nnoremap <buffer> <localleader>lma :Neomake pylint<CR>
  nnoremap <buffer> <silent> <localleader>lai :call AutoImport(expand('<cword>'))<CR>
  nnoremap <buffer> <localleader>lab obreakpoint()<C-c>
  nnoremap <buffer> gd :call LanguageClient#textDocument_definition()<CR>
endfunction

autocmd FileType python :call PythonSpecifics()
" let g:polyglot_disabled = ['python-indent']
let g:python_pep8_indent_searchpair_timeout = 20

