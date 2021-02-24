This document describes packet headers and chunk headers in Teeworlds up to
0.6.x and in DDNet.

All sizes in bits.

    packet_header:
        [ 1] flag_compression
        [ 1] flag_request_resend
        [ 1] flag_connless
        [ 1] flag_control
        [ 2] padding
        [10] ack
        [ 8] num_chunks

        FFFF ppAA  AAAA AAAA  nnnn nnnn

NOTE: `padding` must be zeroed, it's incorrectly used as part of the `ack`
field while unpacking in the reference implementation.

    chunk_header_vital:
        [ 1] flag_resend
        [ 1] flag_vital
        [ 6] <----------
        [ 4] padding   |-- size
        [ 4] <----------

        FFss ssss  PPPP ssss


    chunk_header_nonvital:
        [ 1] flag_resend
        [ 1] flag_vital
        [ 6] <----------
        [ 4] sequence  |-- size
        [ 4] <----------
        [ 8] sequence part 2

        FFss ssss  SSSS ssss  SSSS SSSS

In the packed form, the first four bits of sequence correspond to bits 10 to 6,
and the second part corresponds to the bits 8 to 1.

NOTE: The reference implementation just uses bitwise-or to resolve
contradictions in the overlapping bits.
