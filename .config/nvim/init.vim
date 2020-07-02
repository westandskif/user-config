if strlen($NVIM_PYTHON3_HOST_PROG) > 0
    let g:loaded_python_provider = 0
    let g:python3_host_prog = $NVIM_PYTHON3_HOST_PROG
endif

for f in sort(split(glob(expand('<sfile>:p:h') . "/config/*.vim"), '\n'))
    exe 'source' f
endfor
