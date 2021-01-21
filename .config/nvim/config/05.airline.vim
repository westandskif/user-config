let g:airline#parts#ffenc#skip_expected_string='utf-8[unix]'
let g:airline#extensions#virtualenv#enabled=0
function! AirlineInit()
  let g:airline_section_a = ''
  let g:airline_section_b = ''
  let g:airline_section_x = airline#section#create_right(['tagbar', 'gutentags'])
endfunction
autocmd User AirlineAfterInit call AirlineInit()
