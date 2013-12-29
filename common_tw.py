from binascii import crc32

def crc32tw(data, crc=0):
	return crc32(data, crc)
