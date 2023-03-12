set spell spelllang=en_us
set spelloptions=camel
set spellcapcheck=
" :syn match myExCapitalWords +\<[A-Z]\w*\>+ contains=@NoSpell
autocmd FileType * syntax spell toplevel

set spellfile=./en.utf-8.add
