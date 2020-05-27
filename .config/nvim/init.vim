for f in sort(split(glob(expand('<sfile>:p:h') . "/config/*.vim"), '\n'))
    exe 'source' f
endfor
