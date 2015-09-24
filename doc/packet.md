    packet:
        [ 1] flag_compression
        [ 1] flag_request_resend
        [ 1] flag_connless
        [ 1] flag_control
        [ 2] padding
        [10] ack
        [ 8] num_chunks

        FFFF ppAA  AAAA AAAA  nnnn nnnn

`padding` must be zeroed, it's incorrectly used as part of the `ack` field
while unpacking in the reference implementation.
