Introduction
============

The Teeworlds protocol has several where 32-bit signed integers are encoded
using a variable-length encoding. This documents describes this encoding.

Format
======

All sizes are specified in bits, from the highest bit in a byte to the lowest.

    first_byte:
        [1] flag_extend
        [1] flag_sign
        [6] bits

    next_byte:
        [1] flag_extend
        [7] bits

    last_byte:
        [4] padding
        [4] bits

    int:
        first_byte [next_byte [next_byte [next_byte [last_byte]]]]

As specified, an encoded `int` can have a size of 1 to 5 bytes. Some bytes have
the `flag_extend` variable which specifies whether they're followed by another
byte. The bits of the final integer are the `bits` fields combined, with a
little-endian order. If we call our bits 0 to r, with 0 being the least
significant bit, then it looks like this:

    ES54 3210  Ecba 9876  Ejih gfed  Eqpo nmlk  PPPP utsr
      ^^ ^^^^   ^^^ ^^^^   ^^^ ^^^^   ^^^ ^^^^       ^^^^

Always use the least amount of bytes possible to encode a number. The padding
must always be zeroed.

The `flag_sign` specifies that all the bits of the resulting number should be
flipped, including the sign bit.

NOTE: The reference implementation has no problem with accepting an overlong
representation. The reference implementation also interprets the padding as
part of the number, which leads to weird results. It should always be zeroed.

Examples
========

0 is encoded as `0000 0000`. 1 is encoded as `0000 0001`. -1 is encoded as
`0100 0000` (note that the `bits` field is all zeros).

64 is encoded as `1000 0000  0000 0001`.

Unpacker from ddnet source
================================
```cpp
const unsigned char *CVariableInt::Unpack(const unsigned char *pSrc, int *pInOut)
{
	int Sign = (*pSrc>>6)&1;
	*pInOut = *pSrc&0x3F;

	do
	{
		if(!(*pSrc&0x80)) break;
		pSrc++;
		*pInOut |= (*pSrc&(0x7F))<<(6);

		if(!(*pSrc&0x80)) break;
		pSrc++;
		*pInOut |= (*pSrc&(0x7F))<<(6+7);

		if(!(*pSrc&0x80)) break;
		pSrc++;
		*pInOut |= (*pSrc&(0x7F))<<(6+7+7);

		if(!(*pSrc&0x80)) break;
		pSrc++;
		*pInOut |= (*pSrc&(0x7F))<<(6+7+7+7);
	} while(0);

	pSrc++;
	*pInOut ^= -Sign; // if(sign) *i = ~(*i)
	return pSrc;
}
```
