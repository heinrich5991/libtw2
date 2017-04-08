    -> s:info
    <- s:map_change <----------------------+
    [ -> s:request_map_data ]              |
    [ <- s:map_data ]                      |
    -> s:ready                             |
    [ <- g:sv_motd ]                       |
    <- s:con_ready                         |
    -> g:client_start_info                 |
    [ <- g:sv_vote_clear_options ]         |
    [ <- g:sv_tune_params ]                |
    <- g:sv_ready_to_enter                 |
    -> s:enter_game                        |
    ingame --------------------------------+
