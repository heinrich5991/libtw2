This document describes packet headers and chunk headers in Teeworlds 0.7.0+.

All sizes in bits.

    packet7_header:
        [ 2] reserved
        [ 1] flag_connless
        [ 1] flag_compression
        [ 1] flag_request_resend
        [ 1] flag_control
        [10] ack
        [ 8] num_chunks
        [32] token

        RRff ffAA  AAAA AAAA  nnnn nnnn
        TTTT TTTT  TTTT TTTT  TTTT TTTT  TTTT TTTT

    packet7_header_connless:
        [ 2] reserved
        [ 1] flag_connless
        [ 1] flag_compression
        [ 1] flag_request_resend
        [ 1] flag_control
        [ 2] version
        [32] token
        [32] response_token

        RRff ffVV
        TTTT TTTT  TTTT TTTT  TTTT TTTT  TTTT TTTT
        rrrr rrrr  rrrr rrrr  rrrr rrrr  rrrr rrrr

NOTE: `padding` must be zeroed. If `flag_connless` is set, the other flags must
not be set. `flag_control` implies `!flag_compression`.

NOTE: In `packet7_header_connless`, `version` must be set to 1.

    chunk7_header_vital:
        [ 1] flag_resend
        [ 1] flag_vital
        [ 6] <----------
        [ 4] padding   |-- size
        [ 4] <----------

        FFss ssss  PPss ssss


    chunk7_header_nonvital:
        [ 1] flag_resend
        [ 1] flag_vital
        [ 6] <----------
        [ 4] sequence  |-- size
        [ 4] <----------
        [ 8] sequence

        FFss ssss  SSss ssss  SSSS SSSS
