let g:airline_powerline_fonts = 1
let g:airline#parts#ffenc#skip_expected_string='utf-8[unix]'
let g:airline#extensions#virtualenv#enabled=0
function! AirlineInit()
  let g:airline_symbols.colnr = ' ℅:'
  let g:airline_section_a = airline#section#create([])
  let g:airline_section_b = airline#section#create([])
  let g:airline_section_x = airline#section#create_right(['tagbar', 'gutentags'])
endfunction
autocmd User AirlineAfterInit call AirlineInit()
